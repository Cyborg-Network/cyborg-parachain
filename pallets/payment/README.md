# Payment pallet

The Payment pallet allows users to purchase compute hours from service providers, such as **Cyborg**, and enables compute providers (miners) to be rewarded for their resource contributions.


## Overview

The Payment pallet is designed to manage both the **purchase/consumption of compute hours by users** and the **rewarding of miners** based on their CPU, RAM, and Storage usage. It allows administrators (root users) to set the price for compute services and designate the service provider's account that will receive payments. Users can purchase compute hours using their on-chain balance and consume those hours for services provided by the system.

Additionally, the pallet tracks individual miner resource usage and compensates them accordingly using location-based rates.

Key functionalities provided by the Payment pallet include:

* Setting the price per compute hours
* Assigning a service provider treasury account
* Purchasing compute hours
* Consuming compute hours
* Rewarding miners for CPU, RAM, and Storage contributions
* Distributing pending rewards to miners

The pallet interacts with the system's currency system to manage balances and transfers, ensuring that users pay for the compute hours and that the service provider, compute providers are properly compensated.

To use it in your runtime, you need to implement
[`payment::Config`](https://example.com/dummy-link).

The supported dispatchable functions are documented in the
[`payment::Call`](https://example.com/dummy-link)

### Terminology

* **Compute Hours:**  A measurement unit representing the amount of computational power a user can access over a specified period.
* **Root User:**  The root users refers to an administrator with elevated priviledges within the blockchain system.
* **Treasury Account:**  A special on-chain account that holds funds used for various purposes within the blockchain ecosystem. The treasury account may receive a portion of the fee, payments, rewards from transactions.
* **Miner:** A compute provider contributing hardware resources to process workloads.
* **Reward Rates:** Hourly payout rates for CPU, RAM, and Storage usage, determined externally and passed into the pallet via an oracle or administrative input.

## Interface

### Permissionless Dispatchables

- `purchase_compute_hours`: Allows a user to purchase compute hours. Calculates total cost based on the current price and deducts it from the user’s balance.
- `consume_compute_hours`: Allows a user to consume previously purchased hours. The pallet checks available hours before proceeding.

### Permissioned Dispatchables

- `set_price_per_hour`: Allows the root user to set or update the on-chain price per compute hour.
- `set_service_provider_account`: Allows the root user to assign or update the service provider’s receiving account.
- `record_usage`: Allows a miner to report their percentage usage of CPU, RAM, and Storage (0–100% for each). These values are stored for later reward calculation.
- `reward_miner`: Allows the root user to calculate and store the pending reward for a miner based on their usage and externally provided hourly rates.
- `distribute_rewards`: Allows the root user to distribute all accumulated miner rewards from the service provider’s balance.

## Example Workflow for Miner Rewards

1. **Miner calls `record_usage(cpu, ram, storage)`**  
   → Stores the usage percentages for reward calculation.

2. **Admin calls `reward_miner(hours_worked, miner, cpu_rate, ram_rate, storage_rate)`**  
   → Calculates payout and adds to miner’s pending rewards.

3. **Admin calls `distribute_rewards()`**  
   → Transfers all pending rewards to miners from the service provider account.

_None available._

License: Apache-2.0
