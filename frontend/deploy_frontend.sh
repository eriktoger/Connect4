#!/bin/bash
HASH=$(git rev-parse HEAD;)
C4_API_ROUTE=https://connect4-jixn.onrender.com;
trunk build --release && netlify deploy --prod --dir build --message $HASH;
C4_API_ROUTE=http://localhost:8000