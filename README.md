# RUST/Actix OAuth2 testing

[Read more article](https://www.djamware.com/post/6836f7bc3069a919de614b05/rest-api-security-with-rust-mongodb-and-oauth2)

## Features

- ✅ User registration with password hashing (Argon2)
- ✅ Login that creates JWT access and refresh tokens
- ✅ JWT verification and route protection middleware
- ✅ Refresh token re-generation and persistence
- Swagger-UI 기능 추가

## Testing
Server will run on http://localhost:8080.

## API Endpoints

| Method | Endpoint           | Description          |
| ------ | ------------------ | -------------------- |
| POST   | `/register`        | Register a new user  |
| POST   | `/login`           | Login and get JWTs   |
| POST   | `/refresh`         | Refresh access token |
| GET    | `/api/profile`     | JWT-protected route  |
| POST   | `/logout`          | reset refresh token  |

## Project Structure

```graphql
src/
│
├── handlers/       # Route handlers
├── middleware/     # JWT authentication middleware
├── models/         # Data models
├── utils/          # Utilities like JWT and password hashing
├── main.rs         # Entry point
├── config.rs       # Environment setup
```
