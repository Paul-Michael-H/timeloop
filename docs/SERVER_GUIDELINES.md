# Worldkeeper Development Server Guidelines 🦀

**CRITICAL**: This project exclusively uses **Rust web services** for all server functionality. 

## ⚠️ PROHIBITED TECHNOLOGIES

**DO NOT USE** any of the following for game development:

### ❌ **Node.js Servers**
- No Express.js, Koa, Fastify, or any Node.js web frameworks
- No `http-server`, `live-server`, or Node.js static file servers
- No `npm serve` or similar Node.js tools

### ❌ **Python Servers** 
- No `python -m http.server`
- No Flask, Django, FastAPI, or any Python web frameworks
- No Python static file servers

### ❌ **Other Languages**
- No PHP servers (Apache, Nginx with PHP)
- No Go servers
- No Java servers (Tomcat, Spring Boot, etc.)
- No .NET/C# servers

## ✅ APPROVED RUST SERVERS

### **Primary: API Server** (Recommended)
```bash
# Set JWT secret and run the main API server
$env:JWT_SECRET = "demo-secret-key-for-testing"
cargo run --bin api_server
```
- **URL**: `http://localhost:8081`
- **Features**: Full API endpoints + static file serving
- **Static Files**: Serves from `webclient/static/` 
- **Technology**: Axum + tower-http
- **Purpose**: Production-like environment with authentication

