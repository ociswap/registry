# Registry Blueprint

## Overview

The Registry system is designed to manage and synchronize protocol fees across various pools. It ensures that the fees collected are distributed according to a predefined share and are synchronized over specified intervals to avoid hot shard bottlenecks further down the road.

## Configuration Validation

### Protocol Fee Share

The protocol fee share represents the fraction of the total collected fees that is allocated to the protocol itself. This is a crucial parameter as it directly affects the revenue model of the protocol. The system enforces a maximum limit on the protocol fee share to prevent disproportionately high charges that could deter pool participation. The maximum allowed fee share is set to 25%, ensuring that the majority of the collected fees remain with the pool participants while still providing revenue to the protocol.

### Synchronization Period and Slots

The synchronization period and slots are parameters that control the timing and frequency of fee synchronization across pools.

- **Synchronization Period**: This parameter defines the total duration over which the fees are synchronized. It is crucial that this period is greater than zero to establish a valid operational timeframe for synchronization.

- **Synchronization Slots**: These are subdivisions of the synchronization period. Each slot represents a window in which a specific pool can synchronize its fees. The number of slots must be a positive number and cannot exceed the synchronization period to ensure that each slot is a meaningful duration and that all slots fit within the overall period.

## Synchronization Logic

### Calculation of Next Synchronization Time

The next synchronization time for a pool is calculated based on the current time, the synchronization period, and the number of slots. This calculation ensures that each pool has a designated time slot that does not overlap with others, thus distributing the network load and preventing bottlenecks.

The system calculates the nearest past synchronization cycle based on the current time and the synchronization period. It then determines the specific slot for the pool using a hash function on the pool's address, ensuring that the slot assignment is both deterministic and evenly distributed among all pools.

The next synchronization time is set to the start of the next period after the calculated slot time. If this time is less than one period away from the current time, it is further adjusted to ensure that there is always at least one full period between synchronizations for each pool.

## Fee Collection and Updating Configuration

### Fee Collection Mechanism

When pools collect fees, they deposit them into the Registry. The Registry must handle different types of tokens, and thus it uses a key-value store where each token type is associated with a vault. When a pool deposits fees, the Registry checks if a vault for that specific token type already exists. If not, it creates a new vault for that token type and deposits the fees. If a vault already exists, the fees are added to the existing vault.

### Configuration Updates

The owner of the Registry has the authority to update the configuration parameters (protocol fee share, synchronization period, and slots). Whenever an update is made, the system revalidates the new parameters to ensure they comply with the defined constraints, such as the maximum protocol fee share and the logical consistency of the synchronization period and slots. This revalidation is crucial to maintain the integrity and operational efficiency of the fee management system.

## Conclusion

The Registry system is designed with robust mechanisms to ensure fair and efficient management of protocol fees. By enforcing limits on fee shares and ensuring logical consistency in synchronization parameters, the system maintains a balance between revenue generation for the protocol and equitable fee distribution among pool participants.