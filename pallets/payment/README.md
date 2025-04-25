# Payment pallet

The **Payment Pallet** powers the compute subscription economy on the **Cyborg Parachain**. It enables users to subscribe to compute hours and allows registered miners (compute providers) to earn rewards based on resource usage (CPU, RAM, Storage) both during active and idle states.

## Overview

The Payment pallet is designed to manage both the **subscription/consumption of compute hours by users** and the **rewarding of miners** based on their CPU, RAM, and Storage usage. It allows administrators (root users) to set the price for compute services and designate the service provider's account that will receive payments. Users can subscribe  using their on-chain balance and consume those hours for services provided by the system.

Additionally, the pallet tracks individual miner resource usage and compensates them accordingly using location-based rates.

## 📘 Overview

This pallet manages:

- **Subscription-based compute usage** where users pay upfront to purchase hours.
- **Tracking of compute hour consumption.**
- **Miner rewards**, calculated using usage percentages (CPU, RAM, Storage) and location-based rates.
- **Support for dynamic and static reward rates** for idle and active states.
- **Service provider configuration** for centralized reward distribution.

---

### 📚 Terminology

| Term                     | Meaning                                                                 |
|--------------------------|-------------------------------------------------------------------------|
| **Compute Hours**        | Time purchased by users to access compute services.                     |
| **Subscription Fee**     | Per-hour cost of compute, set by the root user.                         |
| **Service Provider**     | The centralized account that collects user fees and distributes rewards.|
| **Active Usage**         | CPU/RAM/Storage usage reported by a miner during active computation.    |
| **Idle Reward**          | Compensation to miners for being available, regardless of usage.        |
| **Reward Rates**         | Hourly payout rates (per resource) for active and idle modes.           |

---

## 🛠️ Permissionless Dispatchables

These calls can be invoked by any user or registered miner:

- `consume_compute_hours(hours)`: Consume purchased compute hours.
- `subscribe(hours)`: Purchase compute hours by paying tokens upfront.
- `add_hours(extra_hours)`: Add more compute hours to an existing subscription.
- `record_usage(cpu, ram, storage)`: (Miner only) Record current usage in percentages.

---

## 🔒 Permissioned Dispatchables

These calls are **restricted to root origin (sudo/admin)**:

- `set_service_provider_account(account)`: Set the centralized service provider account.
- `set_subscription_fee_per_hour(fee)`: Set the cost per compute hour.
- `set_reward_rates_for_miner(miner, active, idle)`: Define active and idle reward rates per miner.
- `reward_miner(active_hours, idle_hours, miner)`: Calculate and queue rewards for a miner.
- `distribute_rewards()`: Transfer pending miner rewards from the service provider account.

---

## 🔁 Example Workflow for Miner Rewards

1. **User Subscribes**
   - User calls `subscribe(10)` to buy 10 compute hours.
   - Funds are transferred to the Service Provider account.
   - Event: `ConsumerSubscribed`.

2. **User Consumes Compute**
   - User calls `consume_compute_hours(2)` to use 2 hours.
   - Event: `HoursConsumed`.

3. **Miner Records Usage**
   - Miner calls `record_usage(cpu=80, ram=65, storage=90)` to log their contribution.
   - Event: `MinerUsageRecorded`.

4. **Admin Calculates Rewards**
   - Admin calls `reward_miner(active_hours=2, idle_hours=1, miner)`.
   - Rewards are calculated based on recorded usage and reward rates.
   - Event: `MinerRewarded`.

5. **Admin Distributes Rewards**
   - Admin calls `distribute_rewards()`.
   - Rewards are transferred from service provider to miner wallets.
   - Event: `MinerRewarded`.

---


## ✨ Features

- 💳 **Subscription System**: Users can buy compute hours upfront using their on-chain balance.
- 🧠 **Usage-Based Rewards**: Miners earn rewards based on their reported resource usage percentages.
- 💤 **Idle Rewards**: Even idle miners are compensated with default or configured idle rates.
- 🔄 **Flexible Admin Control**: Admin can set subscription fees, miner-specific reward rates, and service provider accounts.
- 📊 **Metered Usage**: Tracks how much compute each user has left and records miner utilization.
- 🔒 **Edge Connect Integration**: Only verified miners (via `pallet-edge-connect`) can report usage and earn rewards.

---

## 🏗️ Storage Overview

| Storage Item                 | Description                                                                 |
|-----------------------------|-----------------------------------------------------------------------------|
| `SubscriptionFee`           | Global per-hour fee for compute subscriptions.                             |
| `ComputeHours`              | Maps user accounts to remaining compute hours.                             |
| `ServiceProviderAccount`    | Account that receives payments from users.                                 |
| `MinerUsage`                | Stores (cpu, ram, storage)% usage for miners.                              |
| `MinerPendingRewards`       | Accumulated rewards to be paid out to each miner.                          |
| `ActiveRewardRates`         | Resource rates when miners are actively serving compute.                   |
| `IdleRewardRates`           | Resource rates when miners are idle.                                       |

---

## ⚙️ Key Extrinsics

| Function                           | Description                                                                 |
|------------------------------------|-----------------------------------------------------------------------------|
| `subscribe(hours)`                | Users purchase compute hours.                                              |
| `add_hours(extra_hours)`          | Users extend their existing subscription.                                  |
| `consume_compute_hours(hours)`    | Users consume their compute hours.                                         |
| `record_usage(cpu, ram, storage)` | Miners report their hourly usage.                                          |
| `reward_miner(active, idle)`      | Admin rewards a miner based on usage and idle hours.                       |
| `distribute_rewards()`            | Transfers all pending rewards to miners from the provider account.         |
| `set_reward_rates_for_miner()`    | Admin sets custom reward rates for a miner.                                |
| `set_service_provider_account()`  | Admin designates the payment recipient account.                            |
| `set_subscription_fee_per_hour()` | Admin sets global hourly subscription fee.                                 |

---

## 🔐 Permissions

- **Root (Sudo) Only**:
  - Setting provider account
  - Updating reward rates
  - Modifying subscription fee
  - Distributing miner rewards

---

## 🧩 Dependencies

- `pallet_edge_connect`: Used to validate miner registration and identity.
- `frame_support`, `frame_system`: Standard FRAME utilities.
- `sp_runtime`: Arithmetic utilities and traits.

---

## 🧪 Testing

Unit tests for various functionalities are included in the `tests` module. Mock implementations exist to simulate runtime behavior and validate business logic.

---



## 💬 Events

- `ConsumerSubscribed`, `SubscriptionRenewed`
- `HoursConsumed`, `MinerUsageRecorded`, `MinerRewarded`
- `ServiceProviderAccountSet`, `RewardRatesUpdated`
- `SubscriptionFeeSet`

---

## ❗ Errors

- `InsufficientBalance`, `InvalidFee`, `AlreadySubscribed`
- `InsufficientComputeHours`, `SubscriptionExpired`
- `InvalidUsageInput`, `NotRegisteredMiner`
- `RewardRateNotSet`

---

## 📈 Future Improvements

- 🔁 Runtime constants for default idle reward rates.
- 🔄 Integration with Conductor module for fiat billing and orchestration.
- ⏳ Time-based automatic reward distribution using off-chain workers or schedulers.
- ⭐ Miner reputation system to dynamically assess miner status (active vs. idle) based on historical behavior and uptime.


---

To use it in your runtime, you need to implement
[`payment::Config`](https://example.com/dummy-link).

The supported dispatchable functions are documented in the
[`payment::Call`](https://example.com/dummy-link)


_None available._

License: Apache-2.0
