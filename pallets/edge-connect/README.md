# Edge-Connect pallet

A pallet for managing the connection and management of off-chain workers (K3s cluster).

## Overview

The edge-connect pallet is responsible for managing the connected workers withing the system. it provides functionality to register, remove workers lined to to user accounts. A storage map maintains worker details, including status, domain, availability, specification and creation block.

The Edge-Connect pallet provides functionality, including:

* Add a new worker to the system.
* Remove a worker from the system.
* Switches worker visibility on or off

To use it in your runtime, you need to implement
[`edge-connect::Config`](https://example.com/dummy-link).

The supported dispatchable functions are documented in the
[`edge-connect::Call`](https://example.com/dummy-link)

### Terminology

* **K3s:** The lightweight version of Kubernetes for managing containers.
* **Worker Node:** An Application inside a container that runs to handle assigned tasks.

## Interface

### Permissionless dispatchables

* `register_worker`: Registers a worker and initialize it with an inactive status.
* `remove_worker`: Remove a worker from storage an deactivates it.
* `toggle_worker_visibility`: Switches the visibility of a worker between active and inactive.

### Permissioned dispatchables

_None available._

License: Apache-2.0
