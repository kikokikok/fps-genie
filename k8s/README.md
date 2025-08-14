# FPS Genie Kubernetes Deployment

This directory contains Kubernetes manifests for deploying the FPS Genie CS2 analysis system.

## Quick Start with minikube

1. **Start minikube**:
   ```bash
   minikube start --memory=4096 --cpus=2
   ```

2. **Deploy the application**:
   ```bash
   kubectl apply -f k8s/
   ```

3. **Check deployment status**:
   ```bash
   kubectl get pods -n fps-genie
   kubectl get services -n fps-genie
   ```

4. **Access the application**:
   ```bash
   # Port forward to access services locally
   kubectl port-forward -n fps-genie svc/fps-genie-svc 8080:80
   ```

## Quick Start with k0s

1. **Install k0s**:
   ```bash
   curl -sSLf https://get.k0s.sh | sudo sh
   sudo k0s install controller --single
   sudo k0s start
   ```

2. **Setup kubectl**:
   ```bash
   sudo k0s kubeconfig admin > ~/.kube/config
   ```

3. **Deploy the application**:
   ```bash
   kubectl apply -f k8s/
   ```

## Components

### Infrastructure Services
- **PostgreSQL + TimescaleDB**: Time-series database for match data
- **Redis**: Caching and job queues
- **Qdrant**: Vector database for behavioral embeddings

### Application Components
- **Data Pipeline**: Processes CS2 demo files
- **Demo Analyzer**: Analyzes individual demo files
- **Analytics**: Advanced analytics and ML processing

## Manifests

1. **namespace-and-config.yaml**: Namespace, ConfigMap, and Secrets
2. **postgres.yaml**: PostgreSQL StatefulSet with TimescaleDB
3. **redis-qdrant.yaml**: Redis and Qdrant deployments
4. **fps-genie-app.yaml**: Main application deployments

## Configuration

### Environment Variables
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string  
- `QDRANT_URL`: Qdrant connection string
- `RUST_LOG`: Logging level (info, debug, warn, error)
- `ENVIRONMENT`: Runtime environment (kubernetes)

### Secrets
- `postgres-password`: PostgreSQL password
- `qdrant-api-key`: Qdrant API key

**Note**: Default passwords are provided for development. Change these for production deployment.

## Resource Requirements

### Minimum Requirements
- **CPU**: 2 cores
- **Memory**: 4GB RAM
- **Storage**: 20GB

### Recommended for Production
- **CPU**: 4+ cores
- **Memory**: 8+ GB RAM
- **Storage**: 100+ GB (with persistent volumes)

## Scaling

Scale individual components:
```bash
# Scale data pipeline
kubectl scale -n fps-genie deployment/fps-genie-app --replicas=3

# Scale analyzer
kubectl scale -n fps-genie deployment/fps-genie-analyzer --replicas=2
```

## Monitoring

Check application logs:
```bash
# Data pipeline logs
kubectl logs -n fps-genie -l component=data-pipeline

# Analyzer logs  
kubectl logs -n fps-genie -l component=analyzer

# Infrastructure logs
kubectl logs -n fps-genie -l app=postgres
kubectl logs -n fps-genie -l app=redis
kubectl logs -n fps-genie -l app=qdrant
```

## Troubleshooting

### Common Issues

1. **Pods stuck in Pending**: Check resource constraints
   ```bash
   kubectl describe pod -n fps-genie <pod-name>
   ```

2. **Database connection issues**: Verify PostgreSQL is ready
   ```bash
   kubectl exec -n fps-genie -it postgres-0 -- pg_isready -U cs2_user
   ```

3. **Image pull issues**: Ensure container registry access
   ```bash
   kubectl describe pod -n fps-genie <pod-name>
   ```

### Health Checks

The application includes readiness and liveness probes to ensure proper startup and health monitoring.

## Production Considerations

1. **Persistent Storage**: Use persistent volumes for PostgreSQL and Qdrant
2. **Resource Limits**: Adjust resource requests/limits based on workload
3. **Secrets Management**: Use external secret management (e.g., sealed-secrets, external-secrets)
4. **Monitoring**: Add Prometheus metrics and Grafana dashboards
5. **Ingress**: Configure ingress controller for external access
6. **Backup**: Setup automated database backups

## Cleanup

Remove all resources:
```bash
kubectl delete namespace fps-genie
```