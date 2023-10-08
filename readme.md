## Web3 indexer, built in Rust

API that Fetches event logs for specified contract address and block range.

## Pre-requisites

need .env file which contains infura url for ethereum mainnet
`cargo run` will run the app

use postman to call the endpoint `localhost:8000/fetchdata`

provide params for target_address, start_block, end_block. eg:
`http://localhost:8000/fetchdata?start_block=18277200&end_block=18277210&target_address=0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`

## What it does right now

rocket web framework is used to host the api

### fetch (event) data

the route fetchdata fetches event logs from a specified block range.
the function for this route initiates a new tokio run time.

Test successful for small range of blocks. multiple api requests done simultaneously.

```
GET /fetchdata?start_block=18277200&end_block=18277210&target_address=0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2:
    => Matched: GET /fetchdata?<target_address>&<start_block>&<end_block> (fetchdata)
spawning task for blocks 18277200 to 18277202
spawning task for blocks 18277203 to 18277205
spawning task for blocks 18277206 to 18277208
spawning task for blocks 18277209 to 18277210
# matching entries: 139
# matching entries: 237
# matching entries: 153
# matching entries: 113
    => Outcome: Success
    => Response succeeded.

```

For large range, would need to ...

- deal with returning large data set: paginate results? or save to a file and return ipfs address?
- use a db to cache block info to save on API requests
  - structure all APIs to read from db by default. only trigger API calls if needed.

For usability

- try to decode event data, if ABI is easily obtained. eg. etherscan
