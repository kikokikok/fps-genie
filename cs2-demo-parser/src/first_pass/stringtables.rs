use super::read_bits::{Bitreader, DemoParserError};
use crate::first_pass::parser_settings::FirstPassParser;
use crate::second_pass::parser_settings::SecondPassParser;
use csgoproto::CMsgPlayerInfo;
use csgoproto::CsvcMsgCreateStringTable;
use csgoproto::CsvcMsgUpdateStringTable;
use prost::Message;
use snap::raw::Decoder;

#[derive(Clone, Debug)]
pub struct StringTable {
    name: String,
    user_data_size: i32,
    user_data_fixed: bool,
    #[allow(dead_code)]
    data: Vec<StringTableEntry>,
    flags: i32,
    var_bit_counts: bool,
}
#[derive(Clone, Debug)]
pub struct StringTableEntry {
    pub idx: i32,
    pub key: String,
    pub value: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct UserInfo {
    pub steamid: u64,
    pub name: String,
    pub userid: i32,
    pub is_hltv: bool,
}

#[derive(Debug)]
pub struct StringTableParams {
    pub bytes: Vec<u8>,
    pub n_updates: i32,
    pub name: String,
    pub udf: bool,
    pub user_data_size: i32,
    pub flags: i32,
    pub variant_bit_count: bool,
}

impl<'a> FirstPassParser<'a> {
    pub fn update_string_table(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let table = CsvcMsgUpdateStringTable::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;

        let st = self.string_tables.get(table.table_id() as usize).ok_or(DemoParserError::StringTableNotFound)?;
        let params = StringTableParams {
            bytes: table.string_data().to_vec(),
            n_updates: table.num_changed_entries(),
            name: st.name.clone(),
            udf: st.user_data_fixed,
            user_data_size: st.user_data_size,
            flags: st.flags,
            variant_bit_count: st.var_bit_counts,
        };
        self.parse_string_table(params)?;
        Ok(())
    }

    pub fn parse_create_stringtable(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let table = CsvcMsgCreateStringTable::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;

        if !(table.name() == "instancebaseline" || table.name() == "userinfo") {
            return Ok(());
        }
        let bytes = match table.data_compressed() {
            true => snap::raw::Decoder::new()
                .decompress_vec(table.string_data())
                .map_err(|_| DemoParserError::MalformedMessage)?,
            false => table.string_data().to_vec(),
        };
        let params = StringTableParams {
            bytes,
            n_updates: table.num_entries(),
            name: table.name().to_string(),
            udf: table.user_data_fixed_size(),
            user_data_size: table.user_data_size(),
            flags: table.flags(),
            variant_bit_count: table.using_varint_bitcounts(),
        };
        self.parse_string_table(params)?;
        Ok(())
    }
    pub fn parse_string_table(&mut self, params: StringTableParams) -> Result<(), DemoParserError> {
        let mut bitreader = Bitreader::new(&params.bytes);
        let mut idx = -1;
        let mut keys: Vec<String> = vec![];
        let mut items = vec![];

        for _upd in 0..params.n_updates {
            let mut key = "".to_owned();
            let mut value = vec![];

            // Increment index
            match bitreader.read_boolean()? {
                true => idx += 1,
                false => idx += (bitreader.read_varint()? + 1) as i32,
            };
            // Does the value have a key
            if bitreader.read_boolean()? {
                // Should we refer back to history (similar to LZ77)
                match bitreader.read_boolean()? {
                    // If no history then just read the data as one string
                    false => key = key.to_owned() + &bitreader.read_string()?,
                    // Refer to history
                    true => {
                        // How far into history we should look
                        let position = bitreader.read_nbits(5)?;
                        // How many bytes in a row, starting from distance ago, should be copied
                        let length = bitreader.read_nbits(5)?;

                        if position >= keys.len() as u32 {
                            key = key.to_owned() + &bitreader.read_string()?;
                        } else if let Some(s) = &keys.get(position as usize) {
                            if length > s.len() as u32 {
                                key = key.to_owned() + s + &bitreader.read_string()?;
                            } else {
                                key = key.to_owned() + s.get(0..length as usize).unwrap_or("") + &bitreader.read_string()?;
                            }
                        }
                    }
                }
                if keys.len() >= 32 {
                    keys.remove(0);
                }
                keys.push(key.clone());
                // Does the entry have a value
                if bitreader.read_boolean()? {
                    let bits: u32;
                    let mut is_compressed = false;

                    match params.udf {
                        true => bits = params.user_data_size as u32,
                        false => {
                            if (params.flags & 0x1) != 0 {
                                is_compressed = bitreader.read_boolean()?;
                            }
                            if params.variant_bit_count {
                                bits = bitreader.read_u_bit_var()? * 8;
                            } else {
                                bits = bitreader.read_nbits(17)? * 8;
                            }
                        }
                    }
                    value = bitreader.read_n_bytes((bits.checked_div(8).unwrap_or(0)) as usize)?;
                    value = if is_compressed {
                        match Decoder::new().decompress_vec(&value) {
                            Ok(bytes) => bytes,
                            Err(_) => return Err(DemoParserError::MalformedMessage),
                        }
                    } else {
                        value
                    };
                }
                if params.name == "userinfo" {
                    if let Ok(player) = parse_userinfo(&value) {
                        if player.steamid != 0 {
                            self.stringtable_players.insert(player.userid, player);
                        }
                    }
                }
                if params.name == "instancebaseline" {
                    match key.parse::<u32>() {
                        Ok(cls_id) => self.baselines.insert(cls_id, value.clone()),
                        Err(_e) => None,
                    };
                }
                items.push(StringTableEntry { idx, key, value });
            }
        }
        self.string_tables.push(StringTable {
            data: items,
            name: params.name,
            user_data_size: params.user_data_size,
            user_data_fixed: params.udf,
            flags: params.flags,
            var_bit_counts: params.variant_bit_count,
        });
        Ok(())
    }
}
pub fn parse_userinfo(bytes: &[u8]) -> Result<UserInfo, DemoParserError> {
    let player = CMsgPlayerInfo::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;
    Ok(UserInfo {
        is_hltv: player.ishltv(),
        steamid: player.xuid(),
        name: player.name().to_string(),
        userid: player.userid() & 0xff,
    })
}

impl<'a> SecondPassParser<'a> {
    pub fn update_string_table(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let table = CsvcMsgUpdateStringTable::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;
        match self.string_tables.get(table.table_id() as usize) {
            Some(st) => {
                let params = StringTableParams {
                    bytes: table.string_data().to_vec(),
                    n_updates: table.num_changed_entries(),
                    name: st.name.clone(),
                    udf: st.user_data_fixed,
                    user_data_size: st.user_data_size,
                    flags: st.flags,
                    variant_bit_count: st.var_bit_counts,
                };
                self.parse_string_table(params)?;
            }
            None => {
                return Ok(());
            }
        }
        Ok(())
    }
    pub fn parse_create_stringtable(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let table = CsvcMsgCreateStringTable::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;
        let bytes = match table.data_compressed() {
            true => snap::raw::Decoder::new()
                .decompress_vec(table.string_data())
                .map_err(|_| DemoParserError::MalformedMessage)?,
            false => table.string_data().to_vec(),
        };
        let params = StringTableParams {
            bytes,
            n_updates: table.num_entries(),
            name: table.name().to_string(),
            udf: table.user_data_fixed_size(),
            user_data_size: table.user_data_size(),
            flags: table.flags(),
            variant_bit_count: table.using_varint_bitcounts(),
        };
        self.parse_string_table(params)?;
        Ok(())
    }
    pub fn parse_string_table(&mut self, params: StringTableParams) -> Result<(), DemoParserError> {
        let mut bitreader = Bitreader::new(&params.bytes);
        let mut idx = -1;
        let mut keys: Vec<String> = vec![];
        let mut items = vec![];

        for _upd in 0..params.n_updates {
            let mut key = "".to_owned();
            let mut value = vec![];

            // Increment index
            match bitreader.read_boolean()? {
                true => idx += 1,
                false => idx += (bitreader.read_varint()? + 1) as i32,
            };
            // Does the value have a key
            if bitreader.read_boolean()? {
                // Should we refer back to history (similar to LZ77)
                match bitreader.read_boolean()? {
                    // If no history then just read the data as one string
                    false => key = key.to_owned() + &bitreader.read_string()?,
                    // Refer to history
                    true => {
                        // How far into history we should look
                        let position = bitreader.read_nbits(5)?;
                        // How many bytes in a row, starting from distance ago, should be copied
                        let length = bitreader.read_nbits(5)?;

                        if position >= keys.len() as u32 {
                            key = key.to_owned() + &bitreader.read_string()?;
                        } else {
                            let s = &keys[position as usize];
                            if length > s.len() as u32 {
                                key = key.to_owned() + s + &bitreader.read_string()?;
                            } else {
                                key = key.to_owned() + s.get(0..length as usize).unwrap_or("") + &bitreader.read_string()?;
                            }
                        }
                    }
                }
                if keys.len() >= 32 {
                    keys.remove(0);
                }
                keys.push(key.clone());
                // Does the entry have a value
                if bitreader.read_boolean()? {
                    let bits: u32;
                    let mut is_compressed = false;

                    match params.udf {
                        true => bits = params.user_data_size as u32,
                        false => {
                            if (params.flags & 0x1) != 0 {
                                is_compressed = bitreader.read_boolean()?;
                            }
                            if params.variant_bit_count {
                                bits = bitreader.read_u_bit_var()? * 8;
                            } else {
                                bits = bitreader.read_nbits(17)? * 8;
                            }
                        }
                    }
                    value = bitreader.read_n_bytes((bits.checked_div(8).unwrap_or(0)) as usize)?;
                    value = if is_compressed {
                        match Decoder::new().decompress_vec(&value) {
                            Ok(bytes) => bytes,
                            Err(_) => return Err(DemoParserError::MalformedMessage),
                        }
                    } else {
                        value
                    };
                }
                if params.name == "userinfo" {
                    if let Ok(player) = parse_userinfo(&value) {
                        if player.steamid != 0 {
                            self.stringtable_players.insert(player.userid, player);
                        }
                    }
                }
                if params.name == "instancebaseline" {
                    match key.parse::<u32>() {
                        Ok(cls_id) => self.baselines.insert(cls_id, value.clone()),
                        Err(_e) => None,
                    };
                }
                items.push(StringTableEntry { idx, key, value });
            }
        }
        self.string_tables.push(StringTable {
            data: items,
            name: params.name,
            user_data_size: params.user_data_size,
            user_data_fixed: params.udf,
            flags: params.flags,
            var_bit_counts: params.variant_bit_count,
        });
        Ok(())
    }
}
