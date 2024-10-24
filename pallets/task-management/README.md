# Task-Management pallet

A pallet for overseeing task scheduling to remote K3s cluster based on availability and specification.

## Overview

The task-management pallet leverages the worker information to assign tasks to connected workers.
The Task-Management pallet provides functionality, including:

* Task Submission*
* Task Execution*
* Task Completion*
* Task Verification*
* Task Resolution*

To use it in your runtime, you need to implement
[`task-management::Config`](https://example.com/dummy-link).

The supported dispatchable functions are documented in the
[`task-management::Call`](https://example.com/dummy-link)

### Terminology

* **K3s:** The lightweight version of Kubernetes for managing containers.

## Interface

### Permissionless dispatchables

* `task_scheduler`: Schedules a new task by creating a task entry and assigning it to a randomly selected available worker.
* `submit_completed_task`: Enables the assigned worker to submit the result of a completed task for verification.
* `verify_completed_task`: The verifier checks the submitted completed task to determine its correctness by comparing the task result hash.
* `resolve_completed_task`: The assigned resolver reviews and resolves the task in case of a dispute over the task verification.

### Permissioned dispatchables

_None available._

License: Apache-2.0
