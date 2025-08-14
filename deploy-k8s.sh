#!/bin/bash
# Quick deployment script for FPS Genie to Kubernetes

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
K8S_DIR="$SCRIPT_DIR/k8s"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    print_error "kubectl is not installed or not in PATH"
    exit 1
fi

# Check if we can connect to a cluster
if ! kubectl cluster-info &> /dev/null; then
    print_error "Cannot connect to Kubernetes cluster"
    print_warning "Make sure you have:"
    echo "  - minikube started: minikube start --memory=4096 --cpus=2"
    echo "  - k0s running: sudo k0s start && sudo k0s kubeconfig admin > ~/.kube/config"
    echo "  - or another K8s cluster configured"
    exit 1
fi

print_status "Kubernetes cluster detected:"
kubectl cluster-info | head -1

# Function to deploy to kubernetes
deploy() {
    print_status "Deploying FPS Genie to Kubernetes..."
    
    # Apply all manifests
    print_status "Applying Kubernetes manifests..."
    kubectl apply -f "$K8S_DIR/"
    
    print_status "Waiting for deployments to be ready..."
    
    # Wait for PostgreSQL StatefulSet
    print_status "Waiting for PostgreSQL to be ready..."
    kubectl wait --for=condition=ready pod -l app=postgres -n fps-genie --timeout=300s
    
    # Wait for Redis deployment
    print_status "Waiting for Redis to be ready..."
    kubectl wait --for=condition=available deployment/redis -n fps-genie --timeout=180s
    
    # Wait for Qdrant deployment
    print_status "Waiting for Qdrant to be ready..."
    kubectl wait --for=condition=available deployment/qdrant -n fps-genie --timeout=180s
    
    # Wait for application deployments
    print_status "Waiting for FPS Genie application to be ready..."
    kubectl wait --for=condition=available deployment/fps-genie-app -n fps-genie --timeout=300s
    kubectl wait --for=condition=available deployment/fps-genie-analyzer -n fps-genie --timeout=300s
    
    print_success "Deployment completed successfully!"
    
    # Show status
    echo ""
    print_status "Deployment status:"
    kubectl get pods -n fps-genie
    echo ""
    kubectl get services -n fps-genie
}

# Function to show logs
logs() {
    print_status "Showing application logs..."
    echo ""
    echo "=== Data Pipeline Logs ==="
    kubectl logs -n fps-genie -l component=data-pipeline --tail=20
    echo ""
    echo "=== Analyzer Logs ==="
    kubectl logs -n fps-genie -l component=analyzer --tail=20
}

# Function to cleanup
cleanup() {
    print_warning "Removing FPS Genie from Kubernetes..."
    kubectl delete namespace fps-genie --ignore-not-found=true
    print_success "Cleanup completed!"
}

# Function to port forward for local access
forward() {
    print_status "Setting up port forwarding for local access..."
    print_status "Access the application at http://localhost:8080"
    print_warning "Press Ctrl+C to stop port forwarding"
    kubectl port-forward -n fps-genie svc/fps-genie-svc 8080:80
}

# Function to show status
status() {
    print_status "FPS Genie deployment status:"
    echo ""
    echo "=== Namespaces ==="
    kubectl get namespace fps-genie 2>/dev/null || echo "fps-genie namespace not found"
    echo ""
    echo "=== Pods ==="
    kubectl get pods -n fps-genie 2>/dev/null || echo "No pods found in fps-genie namespace"
    echo ""
    echo "=== Services ==="
    kubectl get services -n fps-genie 2>/dev/null || echo "No services found in fps-genie namespace"
    echo ""
    echo "=== Deployments ==="
    kubectl get deployments -n fps-genie 2>/dev/null || echo "No deployments found in fps-genie namespace"
}

# Function to show help
help() {
    echo "FPS Genie Kubernetes Deployment Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  deploy     Deploy FPS Genie to Kubernetes cluster"
    echo "  logs       Show application logs"
    echo "  status     Show deployment status"
    echo "  forward    Setup port forwarding for local access"
    echo "  cleanup    Remove FPS Genie from cluster"
    echo "  help       Show this help message"
    echo ""
    echo "Prerequisites:"
    echo "  - kubectl configured and connected to cluster"
    echo "  - Either minikube, k0s, or other K8s cluster running"
    echo ""
    echo "Quick start:"
    echo "  # Start minikube"
    echo "  minikube start --memory=4096 --cpus=2"
    echo ""
    echo "  # Deploy application"
    echo "  ./deploy-k8s.sh deploy"
    echo ""
    echo "  # Access application"
    echo "  ./deploy-k8s.sh forward"
}

# Main script logic
case "${1:-help}" in
    deploy)
        deploy
        ;;
    logs)
        logs
        ;;
    status)
        status
        ;;
    forward)
        forward
        ;;
    cleanup)
        cleanup
        ;;
    help|*)
        help
        ;;
esac