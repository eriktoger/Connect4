# Play against your friends in Connect Four!

## Start locally

- Install rust
- add env for backend:

  - C4_API_ROUTE=http://localhost:8000.
  - I added it to .bashrc : export C4_API_ROUTE=http://localhost:8000

- cd frontend && trunk serve
- cd backend && cargo run

## Deployed at

https://connect4rust.netlify.app

## How to deploy Frontend:

- Run deploy_frontend.sh in frontend/

## How to deployt Backend:

- Run deploy_backend.sh
- Restart backend: heroku ps:restart web -a connect4rust

## About Frontend

A pure Rust frontend built with Yew. Yew is heavily react inspired. Uses Stylist for css.

## About Backend

The backend is handled by Rocket.

## About Common

Things that are used in both frontend and backend, like the data structs that are sent by the api.
