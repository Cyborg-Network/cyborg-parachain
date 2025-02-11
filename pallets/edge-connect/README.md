# Edge-Connect pallet

A pallet for managing the connection and management of [Cyborg Worker Nodes](https://github.com/Cyborg-Network/Cyborg-worker-node).

### Terminology

* **K3s:** The lightweight version of Kubernetes for managing containers.
* **Cyborg Worker Node:** An Application that manages and executes tasks that were assigned to it via [Cyborg Connect](https://github.com/Cyborg-Network/cyborg-connect) and the [Cyborg Parachain](https://github.com/Cyborg-Network/cyborg-parachain).

## Overview

The edge-connect pallet is responsible for managing the connected Cyborg Worker Nodes withing the system. it provides functionality to register, remove workers lined to to user accounts. A storage map maintains worker details, including status, domain, availability, specification and creation block.

The Edge-Connect pallet provides functionality, including:

* Add a new Cyborg Worker Node to the system.
* Remove a Cyborg Worker Node from the system.
* Switches Cyborg Worker Node visibility on or off

## Interface

### Permissionless dispatchables

* `register_worker`: Registers a worker and initialize it with an inactive status.
* `remove_worker`: Remove a worker from storage an deactivates it.
* `toggle_worker_visibility`: Switches the visibility of a worker between active and inactive.

### Permissioned dispatchables

_None available._

### Storage Items

* `AccountWorkers`: Maps user accounts to their registered worker IDs.
* `WorkerClusters`: Maps worker IDs to a struct representing K3s based workers.
* `ExecutableWorkers`: Maps worker IDs to a struct representing Cyborg Worker Nodes.

License: Apache-2.0
