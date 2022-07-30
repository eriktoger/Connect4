#!/bin/bash
HASH=$(git rev-parse HEAD;)
C4_API_ROUTE=https://connect4rust.herokuapp.com;
trunk build && netlify deploy --prod --dir build --message $HASH;
C4_API_ROUTE=http://localhost:8000