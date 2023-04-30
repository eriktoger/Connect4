# Play against your friends in Connect Four!

## Start locally

- Install rust
- add .env in backend folder:
  - MONGO_URI with your mongoDB connection string
- add export C4_API_ROUTE=http://localhost:8000 to .bashrc
- cd frontend && trunk serve # install trunk with $cargo install --locked trunk
- cd backend && cargo run

## Deployed at

https://connect4rust.netlify.app/

## How to deploy Frontend:

- Run deploy_frontend.sh in frontend/

## How to deployt Backend:

- Log in to your render account and click deploy
- Auto deploy can be enable.

## About Frontend

A pure Rust frontend built with Yew. Yew is heavily react inspired. Uses Stylist for css.

## About Backend

The backend is handled by Rocket.

## About Common

Things that are used in both frontend and backend, like the data structs that are sent by the api.
