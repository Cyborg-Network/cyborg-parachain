## Mining Rewards Pallet

The documentation of the Mining Rewards Pallet (integrated into the payments pallet) can be found here: https://github.com/Cyborg-Network/cyborg-parachain/tree/pallet-neuro-zk/pallets/payment

## Neuro ZK Pallet

The documentation of the Neuro ZK Pallet can be found here: https://github.com/Cyborg-Network/cyborg-parachain/blob/nzk-oracle/pallets/neuro-zk/README.md

## Local Setup 
There are four components required to test the Cyborg Network locally:
- Cyborg Parachain
- Cyborg Miner
- Cyborg Gatekeeper
- Cyborg Connect (dApp Frontend)
- Cyborg Oracle Feeder

Running these requires the following:
- A linux based system
- Npm installed (installation via nvm: https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating)
- Rust toolchain for substrate development (setup explained here: https://docs.substrate.io/install/linux/)
- Zombienet (setup explained later, when needed)
- Docker (installation via apt: https://docs.docker.com/engine/install/ubuntu/#install-using-the-repository)
- Ports available: All ports required for zombienet, additionally 8080, 8081 and 9000 for communication between Cyborg Connect and the Miner / Gatekeeper
- wscat installed (for inference testing)

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
git checkout nzk-oracle
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
This should spawn a local testnetwork that can be inspected via the URL shown by zombienet in the terminal.

To test the Mining Rewards Module (Payments Pallet):
```
cd pallets/payment
cargo test
```

To test the Mining Rewards Module (Payments Pallet):
```
cd pallets/neuro-zk
cargo test
```

### Cyborg Miner
###### Notes
Because we currently don't have a cloud solution integrated that facilitates transport of the task (archive containing the model, zk public input, zk proving key, zk settings file) we need to manually insert an archive containing these into the repo before building the docker image. The archive will be provided during the setup steps.
###### Requirements
- Docker needs to be installed
###### Setup
1. Clone the repository and navigate into the docker directory
```
git clone https://github.com/Cyborg-Network/Cyborg-miner.git
cd Cyborg-miner
git checkout neuro-zk-runtime
```
2. Download the archive that contains the task (same archive that is produced during upload)
- download the model archive from: https://drive.google.com/file/d/1aa6zoFQT053-0OAngesD3SJ5vPllqtAe/view?usp=drive_link
- copy the model archive to `Cyborg-miner/miner/current_task`
- navigate back to the root dir of the repository `Cyborg-miner`
4. Build the Docker image
```
docker build -t cyborg-miner:local .
```
4. Run the docker image
```
docker run -it --network="host" -e PARACHAIN_URL="<DIFFERENT_EVERY_TIME>" -e CYBORG_WORKER_NODE_TEST_IP="127.0.0.1" cyborg-miner:local
```
After running the docker image we need to wait for the parachain to finalize the registration of the miner.
If these steps have been completed, the Cyborg Worker Node is now registered on the parachain and listening for inference tasks that have been assigned to it. 
At this point it is able to perform inference on tract compatible models in the .onnx format that have one definite result. 
The archive that we inserted is a mirror image of the archive that the gatekeeper server (which we will set up soon) produces. Since we don't have a cloud solution integrated yet,
we don't have another way of transport at this point.

### Cyborg Connect
###### Requirements
- npm needs to be installed
- testing account seed added to polkadot.js wallet (the whole block needs to be copied '//Alice' included, also Talisman will reject the seed due to to unconventional format)
- zombienet should ideally already be running
```
bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice
```

The setup for Cyborg Connect is fairly straightforward:
1. Clone the repository and navigate into it and switch to the testing branch
```
git clone https://github.com/Cyborg-Network/cyborg-connect.git
cd cyborg-connect
```
2. Install the dependencies
```
npm install
```
3. Edit the development environment variable `.env.development`
- REACT_APP_PARACHAIN_URL= => current collator websocket address exposed by zombienet (eg. ws://127.0.0.1:43927)
- REACT_APP_PROVIDER_SOCKET= => current collator websocket address exposed by zombienet (eg. ws://127.0.0.1:43927
5. Start the development server
```
npm run start
```
5. Prepare the zombienet parachain testnet for testing
- To test cyborg network locally, make sure that zombienet is already running a local version of the Cyborg Parachain otherwise Cyborg Connect will throw an error when in
 the development environment
- To register our account as an oracle feeder (necessary to run Cyborg Oracle feeder) we will use the link provided by zombienet to access the polkadot.js blockexplorer.
- At this point it is important that our polkadot.js wallet is connected to the blockexplorer with the "Alice" testing account that we previously added.
- Next, in the "Developer/Sudo" section we will select the `oracleMembership` pallet with the `addMember` extrinsic and submit a SUDO call to our testnet, adding the `Eve` testing account (address can be selected from the accounts list) as an oracle feeder.
- To be able to purchase compute hours, we need to submit two other extrinsics, so we will select the `payment` pallet with the `setServiceProviderAccount` extrinsic. Again, we want to submit a SUDO call, setting the `Bob` testing account as the service provider (the account that receives the funds when compute hours are purchased).
- The next call that we want to make is in the same pallet, but this time the `setSubscriptionFeePerHour` extrinsic, where we want to sumbit another SUDO call with arbitraty number that sets the price of a single compute hour. When we submit a payment to purchase compute hours, we can come back here and check that Bob has in fact received the funds from the transaction.

We now have 4 different accounts in the loop, Alice, which is the account adopting the comupte consumer perspective, Dave which is the accounts owning the miner, Bob, which is the payment receiving account and Eve which is the account submitting the worker status updates and zk proof verification updates to the parachain as an oracle feeder.

After these steps, cyborg connect is ready for testing and the parachain has been configured properly.

### Gatekeeper
###### Requirements
- Docker needs to be installed
###### Setup
1. Navigate to the cyborg-connect repository (the gatekeeper binary is stored in the cyborg-connect repository)
```
cd cyborg-connect
```
3. Build the Docker image
```
docker build -t cyborg-gatekeeper:local .
```
4. Run the docker image
```
docker run -it --network="host" --rm -e PARACHAIN_URL="<DIFFERENT_EVERY_TIME>" cyborg-gatekeeper:local
```
After running the docker image the gatekeeper will be ready to take requests from the user to compile .ONNX models into a ZK-friendly format and submit the files required for verification to the parachain.

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
ENV ACCOUNT_SEED="//Eve"
```
3. Build the Docker image
```
docker build -t cyborg-oracle-feeder:local .
```
4. Run the docker image
Again, we are using the `--network="host"` flag to avoid network complications and stay on localhost.
```
docker run --network="host" -e PARACHAIN_URL="<DIFFERENT_EVERY_TIME>" -e CYBORG_TEST_KEY="//Eve" cyborg-oracle-feeder:local
```

After these steps have been completed, the Cyborg Oracle Feeder will start to query the worker that we registered previously for its availability status, transform the responses that it gets into a digestible format and submit the result to the parachain. When registering a worker, it will start out as inactive onchain until its status gets updated by the oracle, which happens every 50 blocks, so even if the oracle feeder submitted the initial value, it will take some time until the worker gets updated. For testing purposes it is still possible to assign tasks to an inactive worker.
Additionally it will pick up on zk proofs that were submitted by the miners, verify them and submit the verification result on chain, where the results from the different feeders are aggregated and the proof verification status is updated.

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

| Service   | Description                                 | Required Input                                                           |
| --------- | ------------------------------------------- | ------------------------------------------------------------------------ |
| NeuroZK   | Performs inference verrifiable by zk SNAKRs | Model in .ONNX format and public input in format digestible by the model |

Since we registered a Worker that is able to run executables we will select that option.
###### Worker Selection
The page following service selection shows a world map and asks for the users location which can be retrieved automatically, if allowed, or entered manually. Once the location is granted by the user, the map shows the users location in white, and the locations of the available nodes that are able to execute the type of task that the user wants to execute. The bar at the bottom shows some additional information about the node, such as:
- Location
- Owner
- Distance to the user

###### Selection of Additional Workers and Purchase of Compute Hours
The page following the worker selection map allows the user to do two things:
- Select additional workers: In case the task that the user wants to execute should run on mutliple workers. The worker that the user selected on the map is pre-selected, but additional nearby workers will be recommended. This is disabled for now, as the Cyborg Parachain currently does not accept tasks with assigned to multiple workers yet.
- Subscribe to the service: To execute tasks, the user will be required to deposit the amount of computational hours that the task in question is expected to consume. Each user has a balance of computational hours of which hours can be allocated towards the execution of a task. The users balance of compute hours can be topped up here. Excess hours that have been allocated toward the execution of a task will be refunded to the users balance. **This is necessary to be able to submit an inference request!**
###### Payment Method
For now, the only available payment method is the native token used in Cyborg Network, but in the future it will be possible to pay with other cryptocurrencies, or even FIAT. This page also shows the total cost of execution and requires the user to accept the terms of service to continue with task execution. For model uploading, we will use an example model and public input that can be obtained here:

Model: 
https://github.com/zkonduit/ezkl/blob/main/examples/onnx/random_forest/network.onnx
Public Input:
https://github.com/zkonduit/ezkl/blob/main/examples/onnx/random_forest/input.json

After subscribing to the service, on the deployment screen we want to input these files and click deploy. This will upload the files to the gatekeeper, which makes the files ZK ready and submits the required data to the parachain.

After submission is complete we want to navigate to `http://localhost:8000/access-compute/dashboard/` to access our deployed inference task.
###### Dashboard
The dashboard is shown, which shows a list of Cyborg Workers that the user is currently occupying, which in this case can only be one. When selecting one of the workers, information about the execution process is shown. The user will be able to access confidential information about the task that was dispatched, like logs or execution result locations. This information is being kept confidential, by having the user sign a timestamp with the users wallet. The worker confirms the users identity and establishes a symetrically encrypted websocket connection between Cyborg Connect and the Worker. Once the indentity of the user has beeen confirmed, the user is able to see the following information:
- status of the task
- logs generated during task execution
- location of the result
- status of the worker
- usage metrics (cpu, ram, storage)
- worker specifications (location, OS, amount of memory, amount of storage, etc.)
- current status in the zk proof cycle

###### Inference
Now that the task has been deployed, inference requests can be sent to the miner via wscat under:
```
wscat -c ws://localhost:3000/inference/0
```
A sample inference request to the model that we just deployed would be:
```
{"input_shapes":[[4]],"input_data":[[0.7871868014335632,0.137956440448761,0.38045984506607056,0.012494146823883057]],"output_data":[[0.02]]}
```
This can just be pasted into the wscat connection and the model will respond with an inference response.

Furthermore, ZK proofs can be requested in the dashboard by clicking the `Request Proof` button, which will prompt the miner to submit a zk proof of the model that it is running and submit it to chain. This in turn will prompt the oracle feeder(s) to pick up that proof, verify it and submit the result back to chain, where it will get aggregated with the results from the other feeders.
The ZK Progress bar will reflect the stage advancements.

`Note that this can only be done as soon as the block with containing the task assignment has been finalized and the Worker Node has picked up the assignment, as it will then set the user who assigned the task as the task owner, granting the user access. If that has not yet happened, the user will not be able to open the lock. Finalizaton can take a while, but for now, you can check if the log can be opened by checking if the Worker Node has begun execution.`

## Known issues
- If the following applications are all running simultaneuosly and are submitting transactions from the same account it can happen that a transaction gets rejected because of an invalid nonce, we are currently implementing a definitive solution to prevent this issue.

