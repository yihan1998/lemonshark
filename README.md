# lemonshark

This repository includes a re-write of the original [bullshark](https://arxiv.org/pdf/2201.05677) paper with the following differences:


| |Original Bullshark| Our Bullshark | Lemonshark |
| ----------- | ----------- |----------- |----------- |
|**Client behaviour**| Client is co-located on the Replica sending the transactions in a loopback manner| Same| Client is co-located on the Replica, sending transactions based on the shard the Replica is responsible for at the given time| 
|**Causally dependant transactions**| NA| NA| A non-co-located client sends transactions for all keys to all replicas; therefore, allowing for potential collisions and rollbacks|

## How stuffs calculated
Some terms:
1. **End-to-end latency**: Timer starts when the client sends the request; timer ends when committed
2. **Consensus latency**: Timer starts when Requests are batched by the replica and forwarded (for reliable broadcast); timer ends when committed. 

Note that timer ends for "commit" depends on the latest necessary vote obtained (last cert for reliable broadcast); Since theres no randomization in message delivery, this should suffice in denoting the average time that "commit" has been achieved. 
