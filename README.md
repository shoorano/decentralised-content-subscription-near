# Decentralised Patreon

## MVP
The end goal of this project is to replicate the Patreon peer-to-peer 
subscription model, but with a rust smart contract on the NEAR blockchain
as the backend / database.

Creators will be able to store content as a `String` that can be anything
from markdown, ready to be parsed on the frontend or a youtube link to a private
video (YouTube is not decentralised obviously, but storing video files is beyond
the scope of this project, so youtube will be used for now).

A profile can be either a consumer or a creator, with creators being able to store
content, and set a price for their content.

## To Do

### Backend
- [x] Build out the data struct and methods for the smart contract
- [ ] Make the subscription methods payable and apply a payment logic
- [ ] Make the contract secure (privatise the required methods)
- [ ] Build out the tests
- [ ] Deploy on testnet

### Frontend
- [ ] Build the frontend design
- [ ] Connect to the contract
- [ ] Build frontend tests with jest