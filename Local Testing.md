## Overview
Cyborg Connect is the entry point to the Cyborg Network, a decentralized edge computing platform. By joining the network, users can either provide computational power to contribute to the network's infrastructure or consume computational resources for task execution. The network is built to support a wide range of use cases, from running Docker-based tasks to executing binary programs.
## Local Setup 
There are four components required to test the Cyborg Network locally:
- Cyborg Parachain
- Cyborg Worker Node (x2)
- Cyborg Connect (dApp Frontend)
- Cyborg Oracle Feeder

Running these requires the following:
- A linux based system
- Npm installed (installation via nvm: https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating)
- Rust toolchain for substrate development (setup explained here: https://docs.substrate.io/install/linux/)
- Zombienet (setup explained later, when needed)
- Docker (installation via apt: https://docs.docker.com/engine/install/ubuntu/#install-using-the-repository)
- Ports available: All ports required for zombienet (relay and parachain nodes are 9944, 9955, 9988), additionally 8080 and 8081 for communication between the Cyborg Worker Node and Cyborg Connect

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
4. Set up zombienet
- Download the zombienet binary from: https://github.com/paritytech/zombienet/releases
- Rename the binary from the name it was downloaded as to "zombienet"
- Create a directory for it and move it there
- Add the following to `~/.bashrc` for ease of access: `export PATH="$PATH:/home/<YOUR_USER>/<YOUR_ZOMBIENET_DIR>"`, `export PATH="$PATH:/home/<YOUR_USER>/<YOUR_CYBORG_PARACHAIN_DIR>/target/release"`
- Source your `.bashrc`
- In the directory containing the zombienet binary run `zombienet setup polkadot`
- Navigate to the Cyborg Parachain directory
5. Run the parachain node with zombienet
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
ENV CYBORG_WORKER_NODE_IPFS_API_URL=https://fuchsia-academic-stoat-866.mypinata.cloud
ENV CYBORG_WORKER_NODE_IPFS_API_KEY=21021fa56da65b48c301
ENV CYBORG_WORKER_NODE_IPFS_API_SECRET=6df0a896d2c37606f53ae39f02333484be86d429a898e7c38fb8e4f67da16cb2
```
3. Build the Docker image
```
docker build -t cyborg-worker-node:local .
```
4. Run the docker image
We will perform this step three separate times, to have three different workers in the network. At least two are required for successful task execution, as the second worker verifies the result of the first worker. If the results of the first and second worker differ, a third worker will resolve the conflict. We can neither use the same account, as verifying execution results with workers that belong to the same account as the original executor would pose a security risk, nor can we use the same IP address, so we will need to pass some additional environment variables.

First worker: ACCOUNT_SEED = `//Bob`, CYBORG_WORKER_NODE_TEST_IP=`127.0.0.1`

Second worker: ACCOUNT_SEED = `//Charlie`, CYBORG_WORKER_NODE_TEST_IP=`192.168.1.101`
```
docker run -it --network="host" --rm -e CYBORG_WORKER_NODE_TEST_IP="<DIFFERENT_EVERY_TIME>" -e ACCOUNT_SEED="<DIFFERENT_EVERY_TIME>" cyborg-worker-node:local /bin/bash
```
5. Register the worker
The `docker run ...` command above will move us to the shell within the docker container, where we need to execute the commands to register the worker:
```
/usr/local/bin/cyborg-worker-node registration --parachain-url "$PARACHAIN_URL" --account-seed "$ACCOUNT_SEED" --ipfs-url "$CYBORG_WORKER_NODE_IPFS_API_URL" --ipfs-api-key "$CYBORG_WORKER_NODE_IPFS_API_KEY" --ipfs-api-secret "$CYBORG_WORKER_NODE_IPFS_API_SECRET"
```
After running this command we need to wait for the Cyborg Parachain to finalize the transaction.

6. Start Agent
By this point you should make sure that port 8080 and 8081 are indeed unoccupied, as we will now run the command that starts the binary responsible for communication between the Worker and Cyborg connect. This step should ONLY be performed on the worker that was registered under the IP address `127.0.0.1`.
```
/usr/bin/supervisord -c /etc/supervisor/conf.d/supervisord.config
```

7. Start mining
Once the transaction has been finalized, we can enter the next command, which will prompt the node to start mining, meaning it will start listening to tasks being assigned to it.
```
/usr/local/bin/cyborg-worker-node startmining --parachain-url "$PARACHAIN_URL" --account-seed "$ACCOUNT_SEED"
```
The output of this command will show you the owner of the worker. Take note of the account number of the worker that is registered with the IP address `127.0.0.1`, in our case, the one with the `//Bob` seedphrase. We will need that later, when testing Cyborg Connect.

This process needs to be repeated one more time for the worker that will verify the result of our execution (except for step 6.)

After these steps have been completed, the Cyborg Worker Node is now registered on the parachain and listening for tasks that have been assigned to it. At this point it is able to execute tasks that have one definite result. A CID pointing to a simple `hello-world` binary  that can be used for testing has been provided in the Usage section.

### Cyborg Connect
###### Requirements
- npm needs to be installed
- testing account seed added to polkadot.js wallet (the whole block needs to be copied '//Alice' included, also Talisman will reject the seed due to to unconventional format)
```
bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice
```

The setup for Cyborg Connect is fairly straightforward:
1. Clone the repository and navigate into it and switch to the testing branch
```
git clone https://github.com/Cyborg-Network/cyborg-connect.git
cd cyborg-connect
git checkout test
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
- To test cyborg network locally, make sure that zombienet is already running a local version of the Cyborg Parachain otherwise Cyborg Connect will throw an error when in
 the development environment
- To register our account as an oracle feeder (necessary to run Cyborg Oracle feeder) we will navigate to the dev mode of Cyborg Connect (via the button at the bottom right, saying "Test Chain"). At this point it is important that our polkadot.js wallet is connected to Cyborg Connect with the "Alice" testing account. Next, in the "Pallet Interactor" section we will select the `oracleMembership` pallet with the `addMember` extrinsic and submit a SUDO call to our testnet, adding the `Eve` testing account (address can be copied from the accounts list above the pallet interactor) as an oracle feeder.
- To be able to purchase compute hours, we need to submit two other extrinsics, so we will select the `payment` pallet with the `setServiceProviderAccount` extrinsic. Again, we want to submit a SUDO call, setting the `Eve` testing account as the service provider (the account that receives the funds when compute hours are purchased). The next call that we want to make is in the same pallet, but this time the `setPricePerHour` extrinsic, where we want to sumbit another SUDO call with arbitraty number that sets the price of a single compute hour. When we submit a payment to purchase compute hours, we can come back here and check that Eve has in fact received the funds from the transaction.

We now have 4 different accounts in the loop, Alice, which is the account adopting the comupte consumer perspective, Bob and Charlie, which are the accounts owning the workers, and Eve which is the account receiving funds if compute hours are purchased and also the account submitting the worker status updates to the parachain as an oracle feeder.

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
ENV ACCOUNT_SEED="//Eve"
```
3. Build the Docker image
```
docker build -t cyborg-oracle-feeder:local .
```
4. Run the docker image
Again, we are using the `--network="host"` flag to avoid network complications and stay on localhost.
```
docker run --network="host" cyborg-oracle-feeder:local
```

After these steps have been completed, the Cyborg Oracle Feeder will start to query the worker that we registered previously for its availability status, transform the responses that it gets into a digestible format and submit the result to the parachain. When registering a worker, it will start out as inactive onchain until its status gets updated by the oracle, which happens every 50 blocks, so even if the oracle feeder submitted the initial value, it will take some time until the worker gets updated. For testing purposes it is still possible to assign tasks to an inactive worker.

## Usage
This section describes how to interact with Cyborg Connect to test Cyborg Network.
To test Cyborg Cetwork locally, make sure that zombienet is already running a local version of the Cyborg Parachain.

#### Provide Compute
The Provide Compute section provides UI for users that contribute compute power to the Cyborg Network, and want to manage or monitor their node(s).
###### Worker Registration
The Cyborg Worker Nodes register themselves, as we have seen in the Worker Node section.
###### Worker Removal
To remove your node from the list of nodes, you can click the delete icon in the `Provide Compute` section. If you want to try that, you should do it at the end of the testing run, with an additional worker, as only workers that belong to the account connected to Cyborg Connect will show up here.

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

Since the worker node retrieves the location of its public IP address when registering, we will only see one dot on the map (as all our nodes have the same public IP address). Here we will need to choose the `Manual` selection option and paste the owner account of the worker that we registered under `127.0.0.1` earlier, along with the ID `0`, as every accounts first worker has the ID `0`. We have to make sure that we select this worker, because `subxt`, the library we are using to submit transactions from the Cyborg Worker Node to the parachain will only allow us to make requests to a `ws` (non `wss`) endpoint if on localhost. That means on local testing only one of the workers we registered can communicate directly with Cyborg Connect, because they communicate on the same ports. 
###### Selection of Additional Workers and Purchase of Compute Hours
The page following the worker selection map allows the user to do two things:
- Select additional workers: In case the task that the user wants to execute should run on mutliple workers. The worker that the user selected on the map is pre-selected, but additional nearby workers will be recommended. This is disabled for now, as the Cyborg Parachain currently does not accept tasks with assigned to multiple workers yet.
- Purchase compute hours: To execute tasks, the user will be required to deposit the amount of computational hours that the task in question is expected to consume. Each user has a balance of computational hours of which hours can be allocated towards the execution of a task. The users balance of compute hours can be topped up here. Excess hours that have been allocated toward the execution of a task will be refunded to the users balance.
###### Payment Method
For now, the only available payment method is the native token used in Cyborg Network, but in the future it will be possible to pay with other cryptocurrencies, or even FIAT. This page also shows the total cost of execution and requires the user to accept the terms of service to continue with task execution. For task execution, there is a testing binary that we uploaded to IPFS that can be used. The CID is: `bafkreicw5qjlocihbchmsctw7zbpabicre2ixq6rfimgw372kch5czh3rq`. To execute the binary on your Worker Node, just paste the CID into the modal prompting for the binary.
###### Dashboard
Once a task has been dispatched for execution, the dashboard is shown, which shows a list of Cyborg Workers that the user is currently occupying, which in this case can only be one. When selecting one of the workers, information about the execution process is shown. The user will be able to access confidential information about the task that was dispatched, like logs or execution result locations. This information is being kept confidential, by having the user sign a timestamp with the users wallet. The worker confirms the users identity and establishes a symetrically encrypted websocket connection between Cyborg Connect and the Worker. Once the indentity of the user has beeen confirmed, the user is able to see the following information:
- status of the task
- logs generated during task execution
- location of the result
- status of the worker
- usage metrics (cpu, ram, storage)
- worker specifications (location, OS, amount of memory, amount of storage, etc.)
## Known issues
- If the following applications are all running simultaneuosly and are submitting transactions from the same account it can happen that a transaction gets rejected due to an invalid nonce, we have not decided on a definitive solution to handle these cases yet
- Sometimes the IPFS gateway will not be responsive in time, in which case the process has to be restarted
