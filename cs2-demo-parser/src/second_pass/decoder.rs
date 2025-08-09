use crate::first_pass::read_bits::{Bitreader, DemoParserError};
use crate::second_pass::decoder::Decoder::*;
use crate::second_pass::variants::Variant;
use ahash::AHashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Decoder {
    QuantalizedFloatDecoder(u8),
    VectorNormalDecoder,
    VectorNoscaleDecoder,
    VectorFloatCoordDecoder,
    Unsigned64Decoder,
    CentityHandleDecoder,
    NoscaleDecoder,
    BooleanDecoder,
    StringDecoder,
    SignedDecoder,
    UnsignedDecoder,
    ComponentDecoder,
    FloatCoordDecoder,
    FloatSimulationTimeDecoder,
    Fixed64Decoder,
    QanglePitchYawDecoder,
    Qangle3Decoder,
    QangleVarDecoder,
    BaseDecoder,
    AmmoDecoder,
    QanglePresDecoder,
    GameModeRulesDecoder,
}
#[derive(Debug, Clone)]
pub struct QfMapper {
    pub idx: u32,
    pub map: AHashMap<u32, QuantalizedFloat>,
}
impl<'a> Bitreader<'a> {
    #[inline(always)]
    pub fn decode(&mut self, decoder: &Decoder, qf_map: &QfMapper) -> Result<Variant, DemoParserError> {
        match decoder {
            NoscaleDecoder => Ok(Variant::F32(f32::from_bits(self.read_nbits(32)?))),
            FloatSimulationTimeDecoder => Ok(Variant::F32(self.decode_simul_time()?)),
            UnsignedDecoder => Ok(Variant::U32(self.read_varint()?)),
            QuantalizedFloatDecoder(qf_idx) => Ok(self.decode_qfloat(*qf_idx, qf_map)?),
            Qangle3Decoder => Ok(Variant::VecXYZ(self.decode_qangle_all_3()?)),
            SignedDecoder => Ok(Variant::I32(self.read_varint32()?)),
            VectorNoscaleDecoder => Ok(Variant::VecXYZ(self.decode_vector_noscale()?)),
            BooleanDecoder => Ok(Variant::Bool(self.read_boolean()?)),
            BaseDecoder => Ok(Variant::U32(self.read_varint()?)),
            CentityHandleDecoder => Ok(Variant::U32(self.read_varint()?)),
            ComponentDecoder => Ok(Variant::Bool(self.read_boolean()?)),
            FloatCoordDecoder => Ok(Variant::F32(self.read_bit_coord()?)),
            StringDecoder => Ok(Variant::String(self.read_string()?)),
            QanglePitchYawDecoder => Ok(Variant::VecXYZ(self.decode_qangle_pitch_yaw()?)),
            QangleVarDecoder => Ok(Variant::VecXYZ(self.decode_qangle_variant()?)),
            VectorNormalDecoder => Ok(Variant::VecXYZ(self.decode_normal_vec()?)),
            Unsigned64Decoder => Ok(Variant::U64(self.read_varint_u_64()?)),
            Fixed64Decoder => Ok(Variant::U64(self.decode_uint64()?)),
            VectorFloatCoordDecoder => Ok(Variant::VecXYZ(self.decode_vector_float_coord()?)),
            AmmoDecoder => Ok(Variant::U32(self.decode_ammo()?)),
            QanglePresDecoder => Ok(Variant::VecXYZ(self.decode_qangle_variant_pres()?)),
            GameModeRulesDecoder => Ok(Variant::U32(self.read_nbits(7)?)),
        }
    }
    pub fn decode_qangle_variant_pres(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];

        let has_x = self.read_boolean()?;
        let has_y = self.read_boolean()?;
        let has_z = self.read_boolean()?;

        if has_x {
            v[0] = self.read_bit_coord_pres()?;
        }
        if has_y {
            v[1] = self.read_bit_coord_pres()?;
        }
        if has_z {
            v[2] = self.read_bit_coord_pres()?;
        }
        Ok(v)
    }

    pub fn read_bit_coord_pres(&mut self) -> Result<f32, DemoParserError> {
        Ok(self.read_nbits(20)? as f32 * 360.0 / (1 << 20) as f32 - 180.0)
    }
    pub fn decode_qfloat(&mut self, qf_idx: u8, qf_map: &QfMapper) -> Result<Variant, DemoParserError> {
        match qf_map.map.get(&(qf_idx as u32)) {
            Some(qf) => Ok(Variant::F32(qf.decode(self)?)),
            None => Err(DemoParserError::MalformedMessage),
        }
    }
    pub fn decode_vector_float_coord(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        for idx in 0..3 {
            v[idx] = self.decode_float_coord()?;
        }
        Ok(v)
    }
    pub fn decode_ammo(&mut self) -> Result<u32, DemoParserError> {
        let ammo = self.read_varint()?;
        if ammo > 0 {
            return Ok(ammo - 1);
        }
        Ok(ammo)
    }
    pub fn decode_vector_noscale(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        for idx in 0..3 {
            v[idx] = self.decode_noscale()?;
        }
        Ok(v)
    }
    pub fn decode_uint64(&mut self) -> Result<u64, DemoParserError> {
        let bytes = self.read_n_bytes(8)?;
        match bytes.try_into() {
            Err(_) => Err(DemoParserError::OutOfBytesError),
            Ok(arr) => Ok(u64::from_ne_bytes(arr)),
        }
    }
    pub fn decode_noscale(&mut self) -> Result<f32, DemoParserError> {
        Ok(f32::from_le_bytes(self.read_nbits(32)?.to_le_bytes()))
    }
    pub fn read_string(&mut self) -> Result<String, DemoParserError> {
        let mut s: Vec<u8> = vec![];
        loop {
            let c = self.read_nbits(8)? as u8;
            if c == 0 {
                break;
            }
            s.push(c);
        }
        Ok(String::from_utf8_lossy(&s).to_string())
    }
    pub fn decode_float_coord(&mut self) -> Result<f32, DemoParserError> {
        Ok(self.read_bit_coord())?
    }
    pub fn decode_simul_time(&mut self) -> Result<f32, DemoParserError> {
        Ok(self.read_varint()? as f32 * (1.0 / 30.0))
    }
    pub fn decode_normal(&mut self) -> Result<f32, DemoParserError> {
        let is_neg = self.read_boolean()?;
        let len = self.read_nbits(11)?;
        let result = (len as f64 * (1.0 / ((1 << 11) as f64) - 1.0)) as f32;
        match is_neg {
            true => Ok(-result),
            false => Ok(result),
        }
    }
    pub fn decode_normal_vec(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        let has_x = self.read_boolean()?;
        let has_y = self.read_boolean()?;
        if has_x {
            v[0] = self.decode_normal()?;
        }
        if has_y {
            v[1] = self.decode_normal()?;
        }
        let neg_z = self.read_boolean()?;
        let prod_sum = v[0] * v[0] + v[1] * v[1];
        if prod_sum < 1.0 {
            v[2] = (1.0 - prod_sum).sqrt();
        } else {
            v[2] = 0.0;
        }
        if neg_z {
            v[2] = -v[2];
        }
        Ok(v)
    }
    pub fn decode_qangle_pitch_yaw(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        v[0] = self.read_angle(32)?;
        v[1] = self.read_angle(32)?;
        v[2] = self.read_angle(32)?;
        Ok(v)
    }
    pub fn decode_qangle_all_3(&mut self) -> Result<[f32; 3], DemoParserError> {
        // Used by aimpunch props (not exposed atm) maybe wrong format? correct number of bits anyhow.
        let mut v = [0.0; 3];
        v[0] = self.decode_noscale()?;
        v[1] = self.decode_noscale()?;
        v[2] = self.decode_noscale()?;
        Ok(v)
    }
    pub fn decode_qangle_variant(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        let has_x = self.read_boolean()?;
        let has_y = self.read_boolean()?;
        let has_z = self.read_boolean()?;
        if has_x {
            v[0] = self.read_bit_coord()?;
        }
        if has_y {
            v[1] = self.read_bit_coord()?;
        }
        if has_z {
            v[2] = self.read_bit_coord()?;
        }
        Ok(v)
    }
    pub fn read_angle(&mut self, n: usize) -> Result<f32, DemoParserError> {
        Ok(self.decode_noscale()? / ((1 << n) as f32))
    }
}

const QFF_ROUNDDOWN: u32 = 1 << 0;
const QFF_ROUNDUP: u32 = 1 << 1;
const QFF_ENCODE_ZERO: u32 = 1 << 2;
const QFF_ENCODE_INTEGERS: u32 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantalizedFloat {
    low: f32,
    high: f32,
    high_low_mul: f32,
    dec_mul: f32,
    offset: f32,
    bit_count: u32,
    flags: u32,
    no_scale: bool,
}

impl QuantalizedFloat {
    // More or less directly translated from here:
    // https://github.com/dotabuff/manta/blob/09a1d60ef77f68eef84b79e9ca519caf76a1f291/quantizedfloat.go
    fn validate_flags(&mut self) {
        if self.flags == 0 {
            return;
        }
        if (self.low == 0.0 && (self.flags & QFF_ROUNDDOWN) != 0) || (self.high == 0.0 && (self.flags & QFF_ROUNDUP) != 0) {
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if self.low == 0.0 && (self.flags & QFF_ENCODE_ZERO) != 0 {
            self.flags |= QFF_ROUNDDOWN;
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if self.high == 0.0 && (self.flags & QFF_ENCODE_ZERO) != 0 {
            self.flags |= QFF_ROUNDUP;
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if self.low > 0.0 || self.high < 0.0 {
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if (self.flags & QFF_ENCODE_INTEGERS) != 0 {
            self.flags &= !(QFF_ROUNDUP | QFF_ROUNDDOWN | QFF_ENCODE_ZERO);
        }
    }
    fn assign_multipliers(&mut self, steps: u32) {
        self.high_low_mul = 0.0;
        let range = self.high - self.low;

        let high: u32;
        if self.bit_count == 32 {
            high = 0xFFFFFFFE;
        } else {
            high = (1 << self.bit_count) - 1;
        }

        let mut high_mul: f32;
        // Xd?
        if range.abs() <= 0.0 {
            high_mul = high as f32;
        } else {
            high_mul = (high as f32) / range;
        }

        if (high_mul * range > (high as f32)) || (((high_mul * range) as f64) > ((high as f32) as f64)) {
            let multipliers = vec![0.9999, 0.99, 0.9, 0.8, 0.7];
            for multiplier in multipliers {
                high_mul = (high as f32) / range * multiplier;
                if (high_mul * range > (high as f32)) || (((high_mul * range) as f64) > (high as f32) as f64) {
                    continue;
                }
                break;
            }
        }
        self.high_low_mul = high_mul;
        self.dec_mul = 1.0 / (steps - 1) as f32;
    }
    pub fn quantize(&mut self, val: f32) -> f32 {
        if val < self.low {
            return self.low;
        } else if val > self.high {
            return self.high;
        }
        let i = ((val - self.low) * self.high_low_mul) as u32;
        self.low + (self.high - self.low) * ((i as f32) * self.dec_mul)
    }
    pub fn decode(&self, bitreader: &mut Bitreader) -> Result<f32, DemoParserError> {
        if self.flags & QFF_ROUNDDOWN != 0 && bitreader.read_boolean()? {
            return Ok(self.low);
        }
        if self.flags & QFF_ROUNDUP != 0 && bitreader.read_boolean()? {
            return Ok(self.high);
        }
        if self.flags & QFF_ENCODE_ZERO != 0 && bitreader.read_boolean()? {
            return Ok(0.0);
        }
        let bits = bitreader.read_nbits(self.bit_count)?;
        Ok(self.low + (self.high - self.low) * bits as f32 * self.dec_mul)
    }
    pub fn new(bitcount: u32, flags: Option<i32>, low_value: Option<f32>, high_value: Option<f32>) -> Self {
        let mut qf = QuantalizedFloat {
            no_scale: false,
            bit_count: 0,
            dec_mul: 0.0,
            low: 0.0,
            high: 0.0,
            high_low_mul: 0.0,
            offset: 0.0,
            flags: 0,
        };

        if bitcount == 0 || bitcount >= 32 {
            qf.no_scale = true;
            qf.bit_count = 32;
            return qf;
        } else {
            qf.no_scale = false;
            qf.bit_count = bitcount;
            qf.offset = 0.0;

            if low_value.is_some() {
                qf.low = low_value.unwrap_or(0.0);
            } else {
                qf.low = 0.0;
            }
            if high_value.is_some() {
                qf.high = high_value.unwrap_or(0.0);
            } else {
                qf.high = 1.0;
            }
        }
        if flags.is_some() {
            qf.flags = flags.unwrap_or(0) as u32;
        } else {
            qf.flags = 0;
        }
        qf.validate_flags();
        let mut steps = 1 << qf.bit_count;

        if (qf.flags & QFF_ROUNDDOWN) != 0 {
            let range = qf.high - qf.low;
            qf.offset = range / (steps as f32);
            qf.high -= qf.offset;
        } else if (qf.flags & QFF_ROUNDUP) != 0 {
            let range = qf.high - qf.low;
            qf.offset = range / (steps as f32);
            qf.low += qf.offset;
        }
        if (qf.flags & QFF_ENCODE_INTEGERS) != 0 {
            let mut delta = qf.high - qf.low;
            if delta < 1.0 {
                delta = 1.0;
            }
            let delta_log2 = delta.log2().ceil();
            let range_2: u32 = 1 << delta_log2 as u32;
            let mut bit_count = qf.bit_count;
            loop {
                if (1 << bit_count) > range_2 {
                    break;
                } else {
                    bit_count += 1;
                }
            }
            if bit_count > qf.bit_count {
                qf.bit_count = bit_count;
                steps = 1 << qf.bit_count;
            }
            qf.offset = range_2 as f32 / steps as f32;
            qf.high = qf.low + (range_2 as f32 - qf.offset);
        }

        qf.assign_multipliers(steps);

        if (qf.flags & QFF_ROUNDDOWN) != 0
            && qf.quantize(qf.low) == qf.low {
                qf.flags &= !QFF_ROUNDDOWN;
            }
        if (qf.flags & QFF_ROUNDUP) != 0
            && qf.quantize(qf.high) == qf.high {
                qf.flags &= !QFF_ROUNDUP
            }
        if (qf.flags & QFF_ENCODE_ZERO) != 0
            && qf.quantize(0.0) == 0.0 {
                qf.flags &= !QFF_ENCODE_ZERO;
            }

        qf
    }
}

impl fmt::Display for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::second_pass::decoder::*;

    #[test]
    fn test_qfloat_new() {
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1_023.968_75,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.031_25,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1_023.968_75,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.031_25,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1_023.968_75,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.031_25,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(5), Some(-4.000000), Some(12.000000));
        let correct = QuantalizedFloat {
            low: -4.00000000000000000000000000000000,
            high: 11.937_5,
            high_low_mul: 16.00000000000000000000000000000000,
            dec_mul: 0.003_921_569,
            offset: 0.062_5,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1.00000000000000000000000000000000,
            high_low_mul: 255.00000000000000000000000000000000,
            dec_mul: 0.003_921_569,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(-8), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.999_969_5,
            high_low_mul: 32768.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.000_030_517_578,
            bit_count: 15,
            flags: 4294967288,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(-8), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.999_969_5,
            high_low_mul: 32768.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.000_030_517_578,
            bit_count: 15,
            flags: 4294967288,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.00000000000000000000000000000000,
            high_low_mul: 1.00000000000000000000000000000000,
            dec_mul: 0.000_977_517_1,
            offset: 1.00000000000000000000000000000000,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(1), None, Some(256.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 255.75,
            high_low_mul: 4.00000000000000000000000000000000,
            dec_mul: 0.000_977_517_1,
            offset: 0.25,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 31.999_878,
            dec_mul: 0.000_003_814_712,
            offset: 0.00000000000000000000000000000000,
            bit_count: 18,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 31.999_878,
            dec_mul: 0.000_003_814_712,
            offset: 0.00000000000000000000000000000000,
            bit_count: 18,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 31.999_878,
            dec_mul: 0.000_003_814_712,
            offset: 0.00000000000000000000000000000000,
            bit_count: 18,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1_023.968_75,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.031_25,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1_023.968_75,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.031_25,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1_023.968_75,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.000_030_518_51,
            offset: 0.031_25,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(4.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 3.984_375,
            high_low_mul: 64.00000000000000000000000000000000,
            dec_mul: 0.003_921_569,
            offset: 0.015_625,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(32, None, None, None);
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.00000000000000000000000000000000,
            high_low_mul: 0.00000000000000000000000000000000,
            dec_mul: 0.00000000000000000000000000000000,
            offset: 0.00000000000000000000000000000000,
            bit_count: 32,
            flags: 0,
            no_scale: true,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(32, None, None, None);
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.00000000000000000000000000000000,
            high_low_mul: 0.00000000000000000000000000000000,
            dec_mul: 0.00000000000000000000000000000000,
            offset: 0.00000000000000000000000000000000,
            bit_count: 32,
            flags: 0,
            no_scale: true,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(32, None, None, None);
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.00000000000000000000000000000000,
            high_low_mul: 0.00000000000000000000000000000000,
            dec_mul: 0.00000000000000000000000000000000,
            offset: 0.00000000000000000000000000000000,
            bit_count: 32,
            flags: 0,
            no_scale: true,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(4), Some(-64.000000), Some(64.000000));
        let correct = QuantalizedFloat {
            low: -64.00000000000000000000000000000000,
            high: 64.00000000000000000000000000000000,
            high_low_mul: 7.992_187_5,
            dec_mul: 0.000_977_517_1,
            offset: 0.00000000000000000000000000000000,
            bit_count: 10,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(4), Some(-64.000000), Some(64.000000));
        let correct = QuantalizedFloat {
            low: -64.00000000000000000000000000000000,
            high: 64.00000000000000000000000000000000,
            high_low_mul: 7.992_187_5,
            dec_mul: 0.000_977_517_1,
            offset: 0.00000000000000000000000000000000,
            bit_count: 10,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(20, Some(4), None, Some(128.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 127.999_88,
            high_low_mul: 8192.00000000000000000000000000000000,
            dec_mul: 0.000_000_953_675_2,
            offset: 0.000_122_070_31,
            bit_count: 20,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(-8), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.996_093_75,
            high_low_mul: 256.00000000000000000000000000000000,
            dec_mul: 0.003_921_569,
            offset: 0.003_906_25,
            bit_count: 8,
            flags: 4294967288,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(20, Some(1), None, Some(256.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 255.999_76,
            high_low_mul: 4096.00000000000000000000000000000000,
            dec_mul: 0.000_000_953_675_2,
            offset: 0.000_244_140_63,
            bit_count: 20,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(2), Some(-25.000000), Some(25.000000));
        let correct = QuantalizedFloat {
            low: -24.951_172,
            high: 25.00000000000000000000000000000000,
            high_low_mul: 20.48,
            dec_mul: 0.000_977_517_1,
            offset: 0.048_828_125,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(2), None, Some(102.3));
        let correct = QuantalizedFloat {
            low: 0.099_902_35,
            high: 102.3,
            high_low_mul: 10.009_774,
            dec_mul: 0.000_977_517_1,
            offset: 0.099_902_35,
            bit_count: 10,
            flags: 2,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(2), None, Some(102.3));
        let correct = QuantalizedFloat {
            low: 0.099_902_35,
            high: 102.3,
            high_low_mul: 10.009_774,
            dec_mul: 0.000_977_517_1,
            offset: 0.099_902_35,
            bit_count: 10,
            flags: 2,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(64.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 63.75,
            high_low_mul: 4.00000000000000000000000000000000,
            dec_mul: 0.003_921_569,
            offset: 0.25,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(256.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 255.00000000000000000000000000000000,
            high_low_mul: 1.00000000000000000000000000000000,
            dec_mul: 0.003_921_569,
            offset: 1.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(100.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 100.00000000000000000000000000000000,
            high_low_mul: 2.55,
            dec_mul: 0.003_921_569,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(12, Some(1), None, Some(2048.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 2_047.5,
            high_low_mul: 2.00000000000000000000000000000000,
            dec_mul: 0.000_244_200_26,
            offset: 0.5,
            bit_count: 12,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(17, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 15.999_878,
            dec_mul: 0.000_007_629_453,
            offset: 0.00000000000000000000000000000000,
            bit_count: 17,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(360.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 360.00000000000000000000000000000000,
            high_low_mul: 0.708_333_3,
            dec_mul: 0.003_921_569,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(360.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 360.00000000000000000000000000000000,
            high_low_mul: 0.708_333_3,
            dec_mul: 0.003_921_569,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(16, Some(1), None, Some(500.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 499.992_37,
            high_low_mul: 131.058_9,
            dec_mul: 0.000_015_259_022,
            offset: 0.007_629_394_5,
            bit_count: 16,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(1), None, Some(1500.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1_499.994_3,
            high_low_mul: 174.762_66,
            dec_mul: 0.000_003_814_712,
            offset: 0.005_722_046,
            bit_count: 18,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(11, None, Some(-1.000000), Some(63.000000));
        let correct = QuantalizedFloat {
            low: -1.00000000000000000000000000000000,
            high: 63.00000000000000000000000000000000,
            high_low_mul: 31.984_375,
            dec_mul: 0.000_488_519_8,
            offset: 0.00000000000000000000000000000000,
            bit_count: 11,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(7, Some(1), None, Some(360.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 357.187_5,
            high_low_mul: 0.355_555_56,
            dec_mul: 0.007_874_016,
            offset: 2.812_5,
            bit_count: 7,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(6, Some(2), None, Some(64.000000));
        let correct = QuantalizedFloat {
            low: 1.00000000000000000000000000000000,
            high: 64.00000000000000000000000000000000,
            high_low_mul: 1.00000000000000000000000000000000,
            dec_mul: 0.015_873_017,
            offset: 1.00000000000000000000000000000000,
            bit_count: 6,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.996_093_75,
            high_low_mul: 256.00000000000000000000000000000000,
            dec_mul: 0.003_921_569,
            offset: 0.003_906_25,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, None, Some(0.100000), Some(10.000000));
        let correct = QuantalizedFloat {
            low: 0.1,
            high: 10.00000000000000000000000000000000,
            high_low_mul: 103.333_336,
            dec_mul: 0.000_977_517_1,
            offset: 0.00000000000000000000000000000000,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(2), None, Some(60.000000));
        let correct = QuantalizedFloat {
            low: 0.234_375,
            high: 60.00000000000000000000000000000000,
            high_low_mul: 4.266_24,
            dec_mul: 0.003_921_569,
            offset: 0.234_375,
            bit_count: 8,
            flags: 2,
            no_scale: false,
        };
        assert_eq!(qf, correct);
    }
}
