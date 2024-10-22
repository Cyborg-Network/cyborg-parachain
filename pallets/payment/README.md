# Payment pallet

Pallet allow users to purchase compute hours from service provider, such as **Cyborg**.

## Overview

The Payment pallet is designed to manage the purchase and consumption of compute hours. it allow administrators (root users) to set the price for compute services and designate the service provider's account that will receive payments. Users can the purchase compute hours using their on-chain balance and consume those hours as needed for services provided by the system.

Key functionalities provided by the Payment pallet include:

* Setting the price per compute hours
* Assigning a service provider treasury account
* Purchasing compute hours
* Consuming compute hours

The pallet interacts with the system's currency system to manage balances and transfers, ensuring that users pay for the compute hours and that the service provider, compute providers are properly compensated.

To use it in your runtime, you need to implement
[`payment::Config`](https://example.com/dummy-link).

The supported dispatchable functions are documented in the
[`payment::Call`](https://example.com/dummy-link)

### Terminology

* **Compute Hours:**  A measurement unit representing the amount of computational power a user can access over a specified period.
* **Root User:**  The root users refers to an administrator with elevated priviledges within the blockchain system.
* **Treasury Account:**  A special on-chain account that holds funds used for various purposes within the blockchain ecosystem. The treasury account may receive a portion of the fee, payments, rewards from transactions.

## Interface

### Permissionless dispatchables

* `set_price_per_hour`: This function allows the administrator (root user) to set or update the price per compute hour. This price is stored on-chain and will be used to calculate the cost when users purchase compute hours.
* `set_service_provider_account`: This function allows the admin (root user) to set or update the service provider's account. The service provider account will receive payments when users purchase compute hours.
* `purchase_compute_hours`: This function allows a regular user to purchase compute hours. The user provides a certain number of hours they want to purchase, and the system calculates the total cost based on the current price per compute hour. The user's balance is deducted accordingly, and the compute hours are credited to their account.
* `consume_compute_hours`: This function allows a user to consume some of their previously purchased compute hours. When a user requests to consume compute hours, the system checks if they have enough available hours, and if so, deducts the specified amount from their balance of compute hours.

### Permissioned dispatchables

_None available._

License: Apache-2.0
