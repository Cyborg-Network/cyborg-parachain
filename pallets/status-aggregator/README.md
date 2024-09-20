# Status Feed Aggregator

This pallet serves as a support pallet to the oracle. It processes feed data utilizing the `T::OnNewData` trait from the oracle pallet onto this pallet. The pallet keeps aggregated data and entries up to a block interval defined by `MaxBlockRangePeriod` in the pallet config.

## Usage Overview

First an oracle data feeder must be register onto the `OracleMembership` pallet to gain `Oracle` feeder access. Afterwards, the feeder can call the `feed_value()` extrinsic on the `Oracle` pallet. 

The `feed_value()` extrinsic takes in a value type of `BoundedVec<(T::OracleKey, T::OracleValue), T::MaxFeedValues>

- `T::MaxFeedValues` limits how many values a feeder can enter in each entry 
- `T::OracleKey`: identifies the specific worker to enter information for
- `T::OracleValue`: represents any data we assign to the worker which will be processesed by this aggregator pallet

From here, the status feed aggregator handles all the heavy lifting to process data for each worker based on the values entered.

## Pallet Logic

Data first arrives into this pallet from the `T::OnNewData` trait implementation. It checks whether an oracle feeder provided information for a specfic worker yet via `SubmittedPerPeriod`. If not, `WorkerStatusEntriesPerPeriod` updates by pushing this data along with the current block number onto a bounded vector for a given worker. 

After each `MaxBlockRangePeriod` interval, the `on_finalize()` hook calculates all data in `WorkerStatusEntriesPerPeriod` via `derive_status_percentages_for_period()`. Afterwards, storage resets for `SubmittedPerPeriod`, `WorkerStatusEntriesPerPeriod`, and `LastClearedBlock` updates.

The results of each calulation per period can be accessed from `ResultingWorkerStatusPercentages` and `ResultingWorkerStatus`.


