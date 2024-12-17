## Overview
Cyborg Connect is the entry point to the Cyborg Network, a decentralized edge computing platform. By joining the network, users can either provide computational power to contribute to the network's infrastructure or consume computational resources for task execution. The network is built to support a wide range of use cases, from running Docker-based tasks to executing binary programs.
## Local Setup 
There are four components required to test the Cyborg Network locally:
- Cyborg Parachain
- Cyborg Worker Node
- Cyborg Connect
- Cyborg Oracle Feeder

Running these requires the following:
- A linux based system
- Npm installed
- Rust toolchain for substrate development
- Zombienet
- Docker

This document will walk through how to set each of them up for local testing and how to test their functionality.

We recommend to set the components up in the same order that is outlined in the document.
### Cyborg Parachain
###### Requirements
- Zombienet
- Rust toolchain for substrate development
###### Setup
1. Clone the repository and navigate into it
```
git clone https://github.com/Cyborg-Network/cyborg-parachain.git
cd cyborg-parachain
```
2. Initialize the oracle submodule
```
git submodule update --init
```
3. Build the parachain node
```
cargo build --release
```
4. Run the parachain node with zombienet
```
zombienet --provider native spawn ./zombienet.toml
```
This should spawn a local testnetwork that can be inspected via https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9988#/explorer


### Cyborg Worker Node
###### Requirements
- Docker needs to be installed
###### Setup
1. Clone the repository and navigate into the docker directory
```
git clone https://github.com/Cyborg-Network/Cyborg-worker-node.git
cd Cyborg-worker-node/docker
```
2. Open the Dockerfile and replace the empty environment variables with the following data
```
ENV PARACHAIN_URL=ws://127.0.0.1:9988
ENV ACCOUNT_SEED="bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"
ENV CYBORG_WORKER_NODE_IPFS_API_URL=https://fuchsia-academic-stoat-866.mypinata.cloud
ENV CYBORG_WORKER_NODE_IPFS_API_KEY=21021fa56da65b48c301
ENV CYBORG_WORKER_NODE_IPFS_API_SECRET=6df0a896d2c37606f53ae39f02333484be86d429a898e7c38fb8e4f67da16cb2
```
3. Build the Docker image
	Since this step will already register the worker, the zombienet parachain testnet will need to be running at this point. We are using the `--network="host"` flag here to avoid having to open additional ports to the docker container, since the worker will be sending requests to the parachain. If the worker needs to be registered again for some reason after this image has been built (for example because the zombienet parachain testnet was restarted), it can be done via the `Provide Compute` section of Cyborg Connect
```
docker build -t cyborg-worker-node:local --network="host" .
```
4. Run the docker image
```
docker run <image-id> --network="host"
```

After these steps have been completed, the Cyborg Worker Node is now registered on the parachain and listening for tasks that have been assigned to it. At this point it is able to execute tasks that have one definite result. A CID pointing to a simple `hello-world` binary  that can be used for testing has been provided in the Usage section.


### Cyborg Connect
###### Requirements
- npm needs to be installed
- testing account seed added to polkadot.js wallet (the whole block needs to be copied '//Alice' included, also Talisman will reject the seed due to to unconventional format)
```
bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice
```

The setup for Cyborg Connect is fairly straightforward:
1. Clone the repository and navigate into it
```
git clone https://github.com/Cyborg-Network/cyborg-connect.git
cd cyborg-connect
```
2. Install the dependencies
```
npm install
```
3. Start the development server
```
npm run start
```
4. Prepare the zombienet parachain testnet for testing
- To test cyborg network locally, make sure that zombienet is already running a local version of the Cyborg Parachain, then when running Cyborg Connect, select `Local Chain` in the selector at the bottom of the screen. This will set the RPC endpoint that Cyborg Connect tries to connect to to the local chain that you are running
- To register our account as an oracle feeder (necessary to run Cyborg Oracle feeder) we will navigate to https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9988#/explorer and select the `Sudo` option in the `Developer` dropdown menu. At this point it is important that our polkadot.js wallet is connected to  https://polkadot.js.org with the testing account. Next we will select the `oracleMembership` pallet with the `addMember` extrinsic and submit a sudo call to our testnet, adding our testing account as an oracle feeder (if it is not preselected, select the testing account as the account Id when submitting the extrinsic)
- To be able to purchase compute hours, we need to submit two other extrinsics, so while staying in the `Sudo` section we previously navigated to, we will select the `payment` pallet with the `setServiceProviderAccount` extrinsic. Again, we want to submit a sudo call, setting our testing account as the service provider (the account that receives the funds when compute hours are purchased). The next call that we want to make is in the same pallet, but this time the `setPricePerHour` extrinsic, where we want to sumbit an arbitraty number that sets the price of a single compute hour

After these steps, cyborg connect is ready for testing and the parachain has been configured properly.


### Cyborg Oracle Feeder
###### Requirements
- Docker needs to be installed
###### Setup
1. Clone the repository and navigate into it
```
git clone https://github.com/Cyborg-Network/cyborg-oracle-feeder.git
cd cyborg-oracle-feeder
```
2. Open the Dockerfile and replace the empty environment variables with the following data
```
ENV PARACHAIN_URL=ws://127.0.0.1:9988
ENV ACCOUNT_SEED="bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"
```
3. Build the Docker image
```
docker build -t cyborg-oracle-feeder:local .
```
4. Run the docker image
	Again, we are using the `--network="host"` flag to avoid network complications and stay on localhost.
```
docker run <image-id> --network="host"
```

After these steps have been completed, the Cyborg Oracle Feeder will start to query the worker that we registered previously for its availability status, transform the responses that it gets into a digestible format and submit the result to the parachain. When registering a worker, it will start out as inactive onchain until its status gets updated by the oracle, which happens every 50 blocks. For testing purposes it is still possible to assign tasks to an inactive worker.
## Usage
This section describes how to interact with Cyborg Connect to test Cyborg Network.

To test Cyborg Cetwork locally, make sure that zombienet is already running a local version of the Cyborg Parachain, then when running Cyborg Connect, select `Local Chain` in the selector at the bottom of the screen. This will set the RPC endpoint that Cyborg Connect tries to connect to to the local chain that you are running.
#### Provide Compute
The Provide Compute section provides UI for users that either want to contribute computational power to the Cyborg Network by adding their own node, or have already done so and want to manage or monitor their node(s).
###### Worker Registration
When clicking the `Add Node` button in the dashboard section, the user will be able to manually register a Cyborg Worker. While the Cyborg Worker Node will register itself automatically when installed via the provided intstallation script, some users might prefer to perform these steps manually. In that case they can manually enter the data about their worker here. Required data points are:

| Field              | Description                                                               |
| ------------------ | ------------------------------------------------------------------------- |
| Worker Type        | Ensures the Cyborg Parachain assigns the correct task type to the worker. |
| Domain             | Records the worker’s IP address for monitoring and data transfer.         |
| Latitutde          | Known worker location facilitates edge computing.                         |
| Longitude          | Known worker location facilitates edge computing.                         |
| CPU (no. of cores) | Specifies the worker’s computational capacity.                            |
| RAM (in bytes)     | Specifies the worker’s computational capacity.                            |
| Storage (in bytes) | Specifies the worker’s computational capacity.                            |

###### Worker Monitoring
- The dashboard page shows the workers that are registered on the Cyborg Parachain and are owned by the user accessing the dashboard.
- The polkadot keypair currently connected via the Talisman or Polkadot.js wallets is used to identify the user and return the corresponding workers.

#### Access Compute
The Access Compute section provides UI for users that want to consume compute on the Cyborg Network.
###### Service Selection
Currently there are two types of services for task execution available:

| Service   | Description                           | Required Input            |
| --------- | ------------------------------------- | ------------------------- |
| IPFS      | Executes binary files stored on IPFS. | IPFS CID (IPFS File Link) |
| CyberDock | Executes Docker tasks.                | Docker Image Name         |

Since we registered a Worker that is able to run executables we will select that option.
###### Worker Selection
The page following service selection shows a world map and asks for the users location which can be retrieved automatically, if allowed, or entered manually. Once the location is granted by the user, the map shows the users location in white, and the locations of the available nodes that are able to execute the type of task that the user wants to execute. The bar at the bottom shows some additional information about the node, such as:
- Location
- Owner
- Distance to the user
###### Selection of Additional Workers and Purchase of Compute Hours
The page following the worker selection map allows the user to do two things:
- Select additional workers: In case the task that the user wants to execute should run on mutliple workers. The worker that the user selected on the map is pre-selected, but additional nearby workers will be recommended
- Purchase compute hours: To execute tasks, the user will be required to deposit the amount of computational hours that the task in question is expected to consume. Each user has a balance of computational hours of which hours can be allocated towards the execution of a task. The users balance of compute hours can be topped up here. Excess hours that have been allocated toward the execution of a task will be refunded to the users balance.
###### Payment Method
For now, the only available payment method is the native token used in Cyborg Network, but in the future it will be possible to pay with other cryptocurrencies, or even FIAT. This page also shows the total cost of execution and requires the user to accept the terms of service to continue with task execution. When executing the task, the parachain will reject the task submission, since we only have one worker registered and there are no other workers available to verify the result. 
###### Dashboard
Once a task has been dispatched for execution, the dashboard is shown, which shows a list of Cyborg Workers that the user is currently occupying. When selecting one of the workers, information about the execution process is shown. The user will be able to access confidential information about the task that was dispatched, like logs or execution result locations. This information is being kept confidential, by having the user sign a timestamp with the users wallet. The worker confirms the users identity and establish a symetrically encrypted websocket connection between Cyborg Connect and the Worker. Once the indentity of the user has beeen confirmed, the user is able to see the following information:
- status of the task
- verification status of the task
- logs generated during task execution
- location of the result
- status of the worker
- usage metrics (cpu, ram, storage)
- worker specifications (location, OS, amount of memory, amount of storage, etc.)
## Known issues
- If the following applications are all running simultaneuosly and are submitting transactions from the same account it can happen that a transaction gets rejected due to an invalid nonce
