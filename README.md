# Decentralised Content Subscription - NEAR

## MVP
The end goal of this project is to build a peer-to-peer subscription model, but with a rust smart contract on the NEAR blockchain as the backend / database.

Creators will be able to store content as a `String` that can be anything from markdown, ready to be parsed on the frontend or a link to a privately hosted media. Storing anything but a `String` as content is likely very difficult and for reasons explored [here](https://github.com/shoorano/decentralised-content-subscription-near/blob/main/blog/07-02-2022.md) it is not preferred.

A profile can be either a consumer or a creator, with creators being able to store content, and set a price for their content.

## To Do

### Backend
- [x] Build out the data struct and methods for the smart contract
- [X] Make the subscription methods payable and apply a payment logic
- [X] Build out the tests
- [X] Deploy on testnet
- [X] Build out integration tests using workspaces-rs
- [ ] Update payment model to that outlined in [blog post](https://github.com/shoorano/decentralised-content-subscription-near/blob/main/blog/07-02-2022.md)

### Frontend
- [ ] Decide on framework (vanilla-js, next.js, vue, react or yew/rocket-rs??)
- [ ] Build the frontend design
- [ ] Connect to the contract
- [ ] Build frontend tests

### Next Steps
- [ ]  Audit / secure the contract 
  - [ ]  add account guards for appropriate get methods
  - [ ]  research other possible areas (re-entrancy etc)
- [ ] Make a parent contract that creates a single contract for a single Profile
  - [ ] Research on chain contract creation and cross-contract calls

* Found a useful [link](https://www.youtube.com/watch?v=wC6CS7js-tc) for basic
contract fundamentals
