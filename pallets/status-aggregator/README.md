# Status-Aggregator pallet

This pallet serves as a support pallet to the oracle. It processes feed data utilizing the `T::OnNewData` trait from the oracle pallet onto this pallet. The pallet keeps aggregated data and entries up to a block interval defined by `MaxBlockRangePeriod` in the pallet config.

## Overview

First an oracle data feeder must be register onto the `OracleMembership` pallet to gain `Oracle` feeder access. Afterwards, the feeder can call the `feed_value()` extrinsic on the `Oracle` pallet.

The `feed_value()` extrinsic takes in a value type of `BoundedVec<(T::OracleKey, T::OracleValue), T::MaxFeedValues>

- `T::MaxFeedValues` limits how many values a feeder can enter in each entry 
- `T::OracleKey`: identifies the specific worker to enter information for
- `T::OracleValue`: represents any data we assign to the worker which will be processesed by this aggregator pallet

From here, the status feed aggregator handles all the heavy lifting to process data for each worker based on the values entered.

**Pallet workflow:**
Data first enters the the pallet through the implementation of the the `T::OnNewData` trait. The system checks whether the oracle feeder has already provided information for a specific worker during the current period by querying the`SubmittedPerPeriod` storage. If no data has been submitted yet for that worker, the pallet updates `WorkerStatusEntriesPerPeriod` by appending the new data, along with the current block number, to a bounded vector corresponding to that worker.

At the end of each`MaxBlockRangePeriod` interval, the `on_finalize()` hook triggers the aggregation of all data stored in `WorkerStatusEntriesPerPeriod` by calling `derive_status_percentages_for_period()`. After the aggregation is completed, the pallet resets the storage for `SubmittedPerPeriod`and `WorkerStatusEntriesPerPeriod`, and updates `LastClearedBlock` to the current block.

The results of each calulation per period are stored in `ResultingWorkerStatusPercentages` and `ResultingWorkerStatus`, which can be accessed to view the aggregated worker status and percentages.

To use it in your runtime, you need to implement
[`status-aggregator::Config`](https://example.com/dummy-link).

The supported dispatchable functions are documented in the
[`status-aggregator::Call`](https://example.com/dummy-link)

### Terminology

* **Oracle:** A mechanism that allow external data(data from the real world or off-chain) to be fed into a blockchain system.
* **Oracle Feeder:** A registered entity that provides data.
* **OracleMembership:** Manages who can submit data.
* **Status Feed Aggregator:** The system that processes and aggregates the data submitted by tghe oracle feeders.

## Interface

### Permissionless dispatchables

_None available._

### Permissioned dispatchables

_None available._

### Helper functions

* `on_new_data`: This function is triggered when new data from an oracle is submitted. The data is processed and stored in this pallet by updating the relevant storage items, such as the worker's status entries, if the oracle feeder has not yet submitted data for the worker during the current period.

* `on_finalize`:  This hook is executed at the end of each block and is responsible for calculating and processing the aggregated worker status data based on the information received during the current period. It is called at regular intervals defined by MaxBlockRangePeriod, after which the storage values are reset for the next period.

* `process_aggregate_data_for_period`: This function processes the aggregated worker data collected during the period. It calculates key metrics, such as the worker's online and availability percentages, and updates the storage with these computed results. This step is critical for determining the final worker status.

* `update_worker_clusters`: This function sends the updated worker status information to other pallets that implement the T::WorkerClusterHandler trait. It ensures that worker clusters across the system are updated with the latest status. After updating the worker data, the function emits an event to signal that the worker's status has been modified.

License: Apache-2.0
