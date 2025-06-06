# Cyborg Network - Milestone 1 Delivery

## ⚠️ Outdated Document ⚠️

This document may no longer be relevant and is kept here for reference. Please refer to [Local Testing](https://github.com/Cyborg-Network/cyborg-parachain/blob/master/Local%20Testing.md#local-setup) for the latest information. 

## Overview

The baseline infrastructure layer of the Cyborg Network parachain is delivered as part of this grant. This includes a sample product line called CyberDock, which allows users to deploy publicized Docker images into our network of edge servers. The executed container results will be verified by parallel execution and Hash equations.

## Delivered Items

### 1. Pallets

**Key Components**
We have developed and submitted two primary pallets:

- **Edge Connect** - Manages the connection and management of off-chain workers (K3s clusters).
- **Task Management** - Oversees task scheduling to remote K3s clusters based on availability and specifications.

**Prototype Overview**

This prototype is a blockchain-based machine learning training module featuring on-chain verification and settlement.
It is designed to simulate real-world scenarios involving multiple virtual machine, accurately representing the interaction between blockchain and off-chian components (K3s clusters). These components coordinate to execute tasks, submit result, and update task execution statuses.

**Chain Workflow**<br><br>
<img width="600" alt="Access Compute" src="assets/diagram/chain-workflow.png"><br>

- **Edge Connect Pallet**
The edge-connect pallet is responsible for managing the connected workers within the system It provides functionality to register (`register_worker`)  and remove (`remove_worker`) workers lined to user accounts. A storage map maintains worker details, including status, IP, Domain, availability, specifications, and creation block.

- **Task Management Pallet**
The task-management pallet leverages the worker information to assign tasks to connected workers. The workflow includes:

    + **Task Submission**: Any account can submit a task using `task_scheduler` extrinsic.

    + **Task Execution**: The task is assigned to an account with registered worker, know as the `executor`.

    + **Task Completion**: Upon completion, the executor submits the `submit_completed_task` extrinsic, including the `TaskId` and the tasks's hash output.

    + **Task Verification**: A `verifier`, another account with a registered worker, re-executes the task and submits their hash output with the `TaskId` using the `verify_completed_task` extrinsic. If the hashes match, the task is marked as `completed`. if not, a `resolver` is assigned.

    + **Task Resolution**: The resolver performs the task and submits a hash output with the same `TaskId` using `resolve_completed_task` extrinsic. The correct task output is determined based on matching hashes. if none match, the task is reassigned to a new executor, disticnt from the previous accounts involved.

- **Task Example:**

    + `hello-world`: - Prints the Docker hello world message.
    + `cyborgnetwork/simple-python:new`: - A sample Python program with arithmetic functions.
    + `cyborgnetwork/flog_loader:latest`: - A loader app to test logs.

**Code Repository**

- [Cyborg Parachain](https://github.com/Cyborg-Network/cyborg-parachain) - The Cyborg Substrate Parachain.
- [Cyborg Connect](https://github.com/Cyborg-Network/cyborg-connect) - The Frontend for our App.

### 2. K3s Cluster

**Key Components**
The K3s Cluster is a lightweight Kubernetes distribution designed to manage and orchestrate containerized within Cyborg Network.
We have developed an automated infrastructure setup for off-chain task execution:

- **Master Node Setup** - The master node is configured to initiate the K3s cluster and managed the deployment of tasks. The `MasterSetup.sh` script is delivered as part of this setup, which automates the installation of K3s on the master node and prepares it to handle task orchestration.
- **Worker Node Setup** - The worker node is configured to running the actual application workloads. The `WorkerSetup.sh` script is provided to automate the process of joining worker nodes to the master node using a secure token, ensuring they are ready to receive and execute tasks as part of K3s cluster.

**Code Repository**

- [Worker](https://github.com/Cyborg-Network/Worker) - The K3s worker configuration for Docker image execution.

## Modes of Testing

There are three different options available for testing CyberDock platform, each catering to different environments and needs. These options will be discussed in detail in the following sections:
- **A. Testing Hosted Front End**: This option involves testing the hosted front end against a pre-configured backend that is already aligned and ready to use.
- **B. Hosted Front End with Azure Servers**: This option tests the hosted front end in combination with Azure Servers, providing a hybrid setup.
- **C. Fully Local Setup with UTM and Local Parachain**:This option is for testing the CyberDock platform on a local machine using UTM and a local parachain.

**Prerequisites - Wallet and Accounts**

Before you begin testing, ensure that you have the necessary wallets and accounts configured.

Only accounts with a minimum balance can execute transactions. The `Alice` account is prefunded with the chain's tokens and will be used to interact with the frontend. Whether you are testing locally or using the hosted version of our chain, you will need to use the `Alice` account.

- **Using the Alice Account**
To use the `Alice` account, switch to it or import it through the seed phrase using your prefered wallet extension:

- **Seed Pharase for Alice**

    ```bash
    bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice
    ```

- **Steps to Add the Alice Account in `Polkadot.js` Wallet Extension:**

    - Click the plus icon to reveal a drop-down menu.
    - Select `Import account from pre-existing seed`.

    <img width="300" alt="Access Compute" src="assets/polkajs/polkajs1.png">

    - Paste the Alice seed phrase and click `Next`.

    <img width="300" alt="Access Compute" src="assets/polkajs/polkajs2.png">

    - Add a name and password, then add the account.

    <img width="300" alt="Access Compute" src="assets/polkajs/polkajs3.png">

    - Once successful, you should see the Alice account listed.

    <img width="300" alt="Access Compute" src="assets/polkajs/polkajs4.png"><br><br>

    Once Alice's account is set up, you can proceed with the different modes of testing.

### Option A: Testing Hosted Front End

**Step 1: Access the Hosted Front End**

- Open your web browser and navigate to the provided URL for the hosted front end.
    URL: https://cyborg-network.github.io/cyborg-connect/

**Step 2: Testing the Workflow**

- Connect your wallet when prompted, ensuring you select the Alice wallet or a another funded account and click `Access Compute`.

    <img width="1000" alt="Access Compute" src="assets/frontend/1.png"><br><br>
- Select the hosted chain.

    <img width="1000" alt="Access Compute" src="assets/frontend/select-hosted.png"><br><br>
- Choose CyberDock from the list of product lines.

    <img width="1000" alt="Choose Service" src="assets/frontend/select-cyberdock.png"><br><br>
- Enter the Docker image name (e.g., hello-world).

    - In the Docker image url section enter any one of the file names in the [task examples](https://github.com/Cyborg-Network/cyborg-parachain/blob/master/INSTRUCTIONS.md#task-examples).

    - Any `app/script/website` published as a public Docker image for `linux/amd64` will execute in this system. E.g., hello-world (prints hello world message)

    <img width="1000" alt="Enter docker image" src="assets/frontend/enter-docker.png">

- Pay the fees.

    <img width="1000" alt="Pay the fees" src="assets/frontend/4.png"><br><br>
- Wait for the task to execute (Loader Screen).

    <img width="1000" alt="Loader Screen" src="assets/frontend/5.png"><br><br>
- View the Node List screen. The tasks should be assigned randomly to an available worker. If you registered another worker, the task might be executed by another worker instead.
    - Click through the workers by selecting the left `Dashboard` tab to see the executed task.

    <img width="1000" alt="Node List Screen" src="assets/frontend/8.png"><br><br>
- Access the Deployment dashboard

    <img width="1000" alt="Deployment dashboard" src="assets/frontend/7.png">

### Option B: Hosted Front End with Azure Server

**Step 1: Prerequisites - Infrastructure Setup**

You will need a local Linux machine and two servers hosted within the same virtual network (e.g., a VPC in the case of AWS).
Operating System: `Ubuntu Debian 20.04 LTS` or higher

When deploying servers from a cloud provider, you have the option to add new servers to an existing network. It is crucial that both the K3s master node and worker node are connected within the same network to ensure seamless load distribution, especially when managing heavier data loads such as machine learning models.

- **Local Machine**: Host the Parachain and Frontend.
- **Server 1**: K3s Master node.
- **Server 2**: K3s Worker node.<br><br>

Below is an example server setup for Cloud Azure.

- **VM Setup On Cloud Azure**
  
    - Under the `Create a VM` section, choose `Ubuntu 20.04 or 22.04 LTS`.

    <img width="700" alt=" " src="https://github.com/user-attachments/assets/99999800-03b8-490c-8e23-bf88e172928c"><br><br>

    - In the customization dashboard, create a new resource group and deploy the master node server. Then create a new deployment for worker node.

    <img width="700" alt=" " src="https://github.com/user-attachments/assets/0f6ca8ba-ad2b-41b6-997f-e254ab1d4235"><br><br>

    - Now, choose the same resource group that was created for the first server.

    <img width="700" alt=" " src="https://github.com/user-attachments/assets/9e661c00-3b5b-4139-ac4b-9185a60f0ca1"><br><br>

    - In the `Networking` tab, check that the VNet is the same as that of the master node, then deploy the second server.

    <img width="700" alt=" " src="https://github.com/user-attachments/assets/54b6ddb3-3004-4b91-9028-97d82d177932"><br><br>

    - Cross-verify the VNet of both deployed servers.

    <img width="700" alt=" " src="https://github.com/user-attachments/assets/a0137e1f-a23d-456d-9cad-45b8b34a5ae4">
    <img width="700" alt=" " src="https://github.com/user-attachments/assets/0c049fd2-5125-461a-94bd-c57c70d32f75"><br><br>

    - Finally, go to the `Networking` tab of your virtual machine and add a new `inbound` rule for `port` `3000`. Then Hit `Create port rule`.

    <img width="700" alt=" " src="assets/worker/edit-inbound-port-rule.png"><br><br>

    - Change only the `Destination port ranges` to `3000` and click `Add` once complete. The name should default to `AllowAnyCustom3000Inbound`. You should now see the new rule within your `Network Settings` dashboard.

    <img width="400" alt=" " src="assets/worker/add-inbound-rule.png"><br><br>

    - Get `Master Node` IP Address

    <img width="700" alt=" " src="https://github.com/user-attachments/assets/edce18ad-eaf8-4ac1-a370-eaa0f3a3db10"><br><br>


**Step 2: Register the Worker to the Blockchain**

Make sure you have the domain or IP address of your master node. 
You will use this to register the worker on-chain so that the blockchain can assign tasks to the IP or domain.

- Head over to our [[Hosted Chain]](https://polkadot.js.org/apps/?rpc=wss://fraa-flashbox-3239-rpc.a.stagenet.tanssi.network#/extrinsics)
- Navigate to the extrinsics tab and select the `edge-connect`.
- Go to domain and tick the option to include it
- Enter your domain along with port `3000` which is used by the K3s Worker node, in the format `<yourIpAddress>:3000`.
    - `<yourIpAddress>`: Replace `<yourIpAddress>` with your you master node's public IP address.
    If you registered a domain for your master node, you can use a domain name (e.g. yourWorker-cloud.com).

        <img width="1000" alt="Choose Service" src="assets/add-ip-and-port2.png"><br><br>

- Submit and sign the transaction with funded account
    - Ensure you sign transaction on wallet.

        <img width="1000" alt="Choose Service" src="assets/add-ip-and-port2-submit.png"><br>
        <img width="1000" alt="Choose Service" src="assets/add-ip-and-port2-submit-approved.png"><br>

- Wait for the transaction to succeed and view it at the block explorer. Congratulations, you've registered your worker on chain!


**Step 3: K3s Cluster Setup**

K3s Workers serve as service providers within the network. These workers connect to the RPC endpoint of the blockchain to receive updates and information regarding task execution.
Each K3s worker setup includes one `master node` and at least one `worker node`. The `master node` supplies its `IP` address or `domain name` to the blockchain, enabling the chain to distribute tasks to it.

Once the `master node` receives instructions from the blockchain, it assigns the tasks to its `worker nodes` for execution. Due to the networking requirements, the `master node` and `worker nodes` should be set up on separate machines, ideally within the same local network.

When setting up servers for the K3s workers, ensure that you use two distinct Ubuntu VMs deployed within the same virtual network to facilitate seamless connectivity via their local IP addresses. For AWS, use a VPC; for Azure, deploy both servers within the same virtual network.

Below is an example setup of a K3s Worker that connects to the local blockchain.

- **Master Node Setup for Cloud Azure**

    - Clone the repository and Install Node.js Dependencies
    ```bash
        # Clone the repository
        git clone   https://github.com/Cyborg-Network/Worker.git

        # Navigate to your project directory
        cd Worker

        # Make sure to checkout the branch for the parachain
        git fetch && git branch -a

        git checkout -b updated-parachain origin/updated-parachain

        # Node.js dependencies
        npm install
    ```
    
    - Environment
    Copy the `.env.example` file and replace the contents:
        - Set `WORKER_ADDRESS` to the address where you register this worker on the Cyborg Network chain.
        - Set `RPC_ENDPOINT` to the correct RPC endpoint of the chain you are testing on.
        ```bash
        cp .env.example .env
        ```

     - Run Master Setup Script
    Execute the `MasterSetup.sh` script. This script performs the following actions:
        - Installs k3s on the master node.
        - Saves the k3s node join token to k3s-node-token.txt
        - Starts the Node.js application that listens for deployment requests on port 3000.

        ```bash
            # Make the MasterSetup.sh script executable
            sudo chmod +x MasterSetup.sh

            # Run the MasterSetup.sh script with elevated privileges
            sudo sh MasterSetup.sh

        ```
- **Worker Node Setup for Cloud Azure**
    - Run Worker Setup Scripts.
    Execute the `WorkerSetup.sh <worker-name> <master-ip> <token>` script 
        - `<worker-name>`: The worker's name (use any name of your choice).
        - `<master-ip>`: The `private IP address` from master node.
        - `<token>`: the join token present in the `k3s-node-token.txt` file. 
    Example:

        ```bash
            # Make the WorkerSetup.sh script executable
            sudo chmod +x WorkerSetup.sh

            # Run the WorkerSetup.sh script with elevated privileges
            sudo sh WorkerSetup.sh worker-node-1  10.0.0.5  K10c8230eebd6c64c5cd5aa1::server:8ce7cae600cd
        ```
    - Check Worker connected
    Execute the following command master node. You should see that there is a master node and one worker node. Upon Successful setup proceed to start registering clusters onchain.

        ```bash 
            kubectl get nodes

        ```
         <img width="700" alt=" " src="/assets/kubnodes.png"><br><br>

### Option C: Fully Local Setup with UTM and Local Parachain

Please check with this [video documentation](https://drive.google.com/file/d/1URMopsQZBgGCsZYqiznWOxwmg9wDIdfH/view?usp=sharing) for full walkthrough of local testing

**Step 1: Prerequisites - Infrastructure Setup**

You will need a local Linux machine and two servers hosted within the same virtual network. For local you can use UTM virtual server.
Operating System: `Ubuntu Debian 20.04 LTS` or higher

- **Local Machine**: Host the Parachain and Frontend.
- **Server 1**: K3s Master node.
- **Server 2**: K3s Worker node.<br><br>

Below is an example server setup for fully local.

- **Setup Local Parachain (Local Machine)**
    - Clone the parachain repository and build
    ```bash
        # Clone the repository
        git clone --recurse-submodules https://github.com/Cyborg-Network/cyborg-parachain.git

        # Or alternatively you can run this command
        git clone https://github.com/Cyborg-Network/cyborg-parachain.git
        git submodule update --init
    ```
    - Compile the node
    ```sh
        # Change Directory to Cyborg-parachain
        cd cyborg-parachain
        
        cargo build --release
    ```
    - Run Tests
    ```sh
        # Execute Cargo Test
        cargo test
    ```
    - 🐳 Alternatively, build the docker image:
    ```sh
        docker build . -t cyborg-parachain
    ```

- **Setup Zombienet (Local Machine)**
    - [Download Zombienet Binary](https://github.com/paritytech/zombienet/releases/tag/v1.3.109)
    - [Donwload Polkadot Libary](https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-stable2407-2)
    - Using Binaries on MacOS
        - Move the binary (`zombienet-macos-arm64`, `polkadot`, `polkadot-prepare-worker`, `polkadot-execute-worker`) to your working directory (e.g, `zombienetbin`)
        - Rename the binary to just `zombienet` without any `macos-<version>` extension.
        ```bash
            # Enable the binary to be executable
            chmod +x ./zombienet

            # Remove the binary from quarantine
            xattr -d com.apple.quarantine ./zombienet
            
            # Edit your shell's configuration to Add Zombienet to your PATH, using $HOME/zombienetbin as the working directory in this example
            export PATH="$HOME/zombienetbin/:$PATH"

            #Reload the configuration file
            # source ~/.zshrc       #for Zsh
            # source ~/.bashrc      #for Bash

            # View zombienet help
            ./zombienet help
        ```        
    - Ensure that parachain node are added to your PATH
        - All above files can be found in `target` folder within cyborg-parachain project.
        ```bash
            export PATH="$HOME/cyborg-parachain/target/release/:$PATH"
            #Reload the configuration file
            # source ~/.zshrc       #for Zsh
            # source ~/.bashrc      #for Bash
        ```
    - Start the local development chain with a single relay chain node and a single parachain collator
        - in this example, the local parachain path is `$HOME/cyborg-parachain`
    ```bash
        #Working Directory
        cd $HOME/cyborg-parachain

        # Start zombienet
        zombienet --provider native spawn ./zombienet.toml
    ```

- **Setup Cyborg Connect (Local Machine)**
    - Clone the front-end application, Cyborg-connect repository and build
    ```bash
        # Clone the repository
        git clone https://github.com/Cyborg-Network/cyborg-connect.git

        # Change to working directory
        cd cyborg-connect

        npm install
    ```
    - You can start in development mode to connect to local running node
    ```sh
        npm run start
    ```
    - You can also build the app in proudction mode
    ```sh
        npm run build
    ```
- **VM Setup On UTM**
  
    - Under the `Create a VM` section, choose `Ubuntu 20.04 or 22.04 LTS`.
    UTM Setting:
        - `Network Mode`: Bridge

    <img width="700" alt=" " src="assets/utm/workervm.png"><br><br>

**Step 2: Register the Worker to the Blockchain**

Make sure you have the domain or IP address of your master node. 
You will use this to register the worker on-chain so that the blockchain can assign tasks to the IP or domain.

- Head over to our [[Local Chain]](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9988#/extrinsics)
- Navigate to the extrinsics tab and select the `edge-connect`.
- Go to domain and tick the option to include it
- Enter your domain along with port `3000` which is used by the K3s Worker node, in the format `<yourIpAddress>:3000`.
    - `<yourIpAddress>`: Replace `<yourIpAddress>` with your you master node's public IP address.
    If you registered a domain for your master node, you can use a domain name (e.g. yourWorker-cloud.com).
    <img width="1000" alt="Choose Service" src="assets/add-ip-and-port-local.png"><br><br>
- Submit and sign the transaction with funded account
    - Ensure you sign transaction on wallet.
- Wait for the transaction to succeed and view it at the block explorer. Congratulations, you've registered your worker on chain!


**Step 3: K3s Cluster Setup**

K3s Workers serve as service providers within the network. These workers connect to the RPC endpoint of the blockchain to receive updates and information regarding task execution.
Each K3s worker setup includes one `master node` and at least one `worker node`. The `master node` supplies its `IP` address or `domain name` to the blockchain, enabling the chain to distribute tasks to it.

Once the `master node` receives instructions from the blockchain, it assigns the tasks to its `worker nodes` for execution. Due to the networking requirements, the `master node` and `worker nodes` should be set up on separate machines, ideally within the same local network.

When setting up servers for the K3s workers, ensure that you use two distinct Ubuntu VMs deployed within the same virtual network to facilitate seamless connectivity via their local IP addresses.

Below is an example setup of a K3s Worker that connects to the local blockchain.

- **Master Node Setup for Local VM**

    - Clone the repository, Install Node.js Dependencies and PM2
    ```bash
        # Clone the repository
        
        # Update current Ubuntu package
        sudo  apt-get update

        # Install dnslookup
        sudo apt-get install dnsutils

        # Install Git
        sudo apt-get install git

        git clone   https://github.com/Cyborg-Network/Worker.git

        # Navigate to your project directory
        cd Worker
        
        # Install nvm (Node Version Manager) for Master Node only
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash

        # Reload bash shell
        source ~/.bashrc

        # download and install Node.js for Master Node only (you may need to restart the terminal).
        nvm install 18

        # Install PM2 on the master node for running the deployment service
        npm i -g pm2

    ```  
    - Environment Setup
    Setup `.env` variables:
        - Set `WORKER_ADDRESS` to the address where you register this worker on the Cyborg Network chain.
        - Set `RPC_ENDPOINT` to the correct RPC endpoint of the chain you are testing on.
        - Set `IP_ADDRESS` to the IP address of your worker (public or private).
        - Set `DOMAIN_NAME` to the domain name IP Address (optional).
    
        Install K3s cluster master node:
        - You can always run the `MasterSetup.sh` again to retrieve a worker node join token (`k3s-node-token.txt`).

        Note: 
        - You can edit the `.env` manually afterwards depending on your setup preference.
        - if you are running Cyborg Node locally (not on vm), update the `RPC_ENDPOINT` to `ws://<your-local-pc-ip-address>:9988`. 
        - You can fetch your local IP address of your computer on the newtwork using `ipconfig getifaddr en0`

        ```bash
            # Setup your envinronment variable defaults for running the worker locally within a local network.
            # This script automates the creation of a `.env` file, set the variable defaults and run the MasterSetup script.

            npm run setup:local
        ```

     - Run Master Setup Script (Optional)
    Execute the `MasterSetup.sh` script. This script performs the following actions:
        - Installs k3s on the master node.
        - Saves the k3s node join token to `k3s-node-token.txt`

        ```bash
            # Make the MasterSetup.sh script executable
            sudo chmod +x MasterSetup.sh

            # Run the MasterSetup.sh script with elevated privileges
            sudo sh MasterSetup.sh

        ```
- **Worker Node Setup for Local VM**
    - Run Worker Setup Scripts.
    Execute the `WorkerSetup.sh <worker-name> <master-ip> <token>` script 
        - `<worker-name>`: The worker's name (use any name of your choice).
        - `<master-ip>`: The `private IP address` from master node.
        - `<token>`: the join token present in the `k3s-node-token.txt` file.
    Example:
        ```bash
            # Make the WorkerSetup.sh script executable
            sudo chmod +x WorkerSetup.sh

            # Run the WorkerSetup.sh script with elevated privileges
            sudo sh WorkerSetup.sh worker-node-1  10.0.0.5  K10c8230eebd6c64c5cd5aa1::server:8ce7cae600cd
        ```
    - Check Worker connected
    Execute the following command master node. You should see that there is a master node and one worker node. Upon Successful setup proceed to start registering clusters onchain.
        ```bash 
            kubectl get nodes

        ```
         <img width="700" alt=" " src="/assets/kubnodes2.png"><br><br>


## Local Development Chain

🧟 This project uses [Zombienet](https://github.com/paritytech/zombienet) to orchestrate the relaychain and parachain nodes.
You can grab a [released binary](https://github.com/paritytech/zombienet/releases/latest) or use an [npm version](https://www.npmjs.com/package/@zombienet/cli).

This template produces a parachain node. You still need a relaychain node. You can download the `polkadot` binaries (and the accompanying `polkadot-prepare-worker` and `polkadot-execute-worker`) from [Polkadot SDK releases](https://github.com/paritytech/polkadot-sdk/releases/latest).

Make sure to bring the parachain node, as well as `polkadot`, `polkadot-prepare-worker`, `polkadot-execute-worker`,
and `zombienet` - into `PATH` like so:

```sh
export PATH="./target/release/:$PATH"
```

This way, we can conveniently use them un the following steps.

👥 The following command starts a local development chain, with a single relay chain node and a single parachain collator:

```sh
zombienet --provider native spawn ./zombienet.toml

# Alternatively, the npm version:
npx --yes @zombienet/cli --provider native spawn ./zombienet.toml
```

### Register on k3s

Make sure you have the domain or IP address of your worker node. You will use this to register the worker on-chain so that the blockchain can assign tasks to the IP or domain.

- Head over to our [[Hosted Chain]](https://polkadot.js.org/apps/?rpc=wss://fraa-flashbox-3239-rpc.a.stagenet.tanssi.network#/extrinsics)
- Navigate to the extrinsics tab and select the `edge-connect`.
- Go to domain and tick the option to include it
- Enter your domain along with port 3000 which is used by the K3s Worker node, in the format `yourIpAddress:3000`.
Replace `yourIpAddress` with your you master node's public IP address. 
If you registered a domain for your master node, you can use a domain name (e.g. yourWorker-cloud.com).

<img width="1000" alt="Choose Service" src="assets/add-ip-and-port.png"><br><br>

- Submit and sign the transaction with a funded account

<img width="1000" alt="Choose Service" src="assets/sign-submit.png"><br><br>

Wait for the transaction to succeed and view it at the block explorer. Congratulations, you've registered your worker on chain!

## Onchain Interaction

### Register a Worker

Go to the [`Polkadotjs Apps`](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9988#/) with the correct websocket port set for the parachain (which should be port 9988). This should also appear in the terminal for zombienet for `alice-1` in the Direct Link section:

<img width="1000" alt="Zombienet terminal view" src="assets/zombinet-collator.png"><br></br>

Once your parachain node starts producing blocks, navigate to the extrinsics tab and select `edge-connect`.

<img width="1000" alt="Polkadotjs App extrinsic workerClusters" src="assets/workerClusters.png"><br><br>

Tick the option to include domain and enter your domain, or tick the option to include an IP/Port.

<img width="1000" alt="Enter API" src="assets/register-domain.png"><br><br>

Then sign and submit the transaction. Congratulations, you've registered your worker on-chain!

### Task Creation

Anyone can register a task onchain. The task must be a valid Docker image that is publically accessable. To create a task, at least one worker must be registered.

To create a task, navigate to the `taskManagement` extrinsic to select the `taskScheduler` function. Enter a valid Docker image in the `taskData` section then sign and submit.

<img width="1000" alt="Create task" src="assets/task-create.png"><br><br>

Go to the explorer section to view which worker called `executor` was assigned the task. This account must complete the task and submit a hash out the task output.

<img width="1000" alt="Assigned worker executor" src="assets/assigned-worker.png"><br><br>

Congratulations! A task was successfully scheduled!

### Task Completion and Verification

For the task to be successfully verified as complete, two more workers need to be registered. One to verify task output and a second in case the first fails the verification. 
Follow the steps above to register two more workers.

Now, from the account assigned the task, navigate to `taskManagement` extrinsic to the `submitCompletedTask` method. Use the `taskId` and enter a hash value.

<img width="1000" alt="Submit completed task" src="assets/assigned-worker.png"><br><br>

Once submitted, navigate to the explorer section to view which worker was assigned as `verifier`.

<img width="1000" alt="Assigned worker verifier" src="assets/assigned-verifier.png"><br><br>

Now, from the `verifier` account, navigate to the `taskManagement` extrinsic and select the `submitCompletedTask` method. Use the `taskId` and enter a hash value.

You can enter in the same hash value as before or a different one. The same hash value will complete the task, while a different hash value will assign a `resolver`.

With the same hash:

<img width="1000" alt="Verify completed task extrinsic" src="assets/verify-task.png"><br><br>

Check the explorer to see an event emitted for the taskId that is verified as complete:

<img width="1000" alt="Verify completed task event" src="assets/task-verified.png"><br><br>

Check chain state in `taskManagment` for `taskStatus` call of the `taskId` to ensure it shows `Completed`.
<img width="1000" alt="Completed task" src="assets/completed-task.png"><br><br>

If the hashes from both `verifier` and `executor` differ,  a worker will be assigned as `resolver`. You can check for this in the explorer section of the events for `VerifierResolverAssigned` event to find the `resolver`.
Following the similar steps as above, you will enter the `taskManagement` extrinsic and select the `resolveCompletedTask` method to enter the `taskId` and a output hash.

# Worker Logs

The results of executed tasks are displayed in the terminal section of the deployment dashboard. Below are examples of worker logs corresponding to the task examples mentioned above.

## Hello World
Docker Image URL: https://hub.docker.com/_/hello-world

    [172.212.108.104:3000][TaskID: 74] Status: 2024-08-06T13:54:01.000Z
    [172.212.108.104:3000][TaskID: 74] Status: ReplicaSet "dynamic-deployment-569b8o-54cc46b66d" has successfully progressed.
    [172.212.108.104:3000][TaskID: 74] Status: NewReplicaSetAvailable
    [172.212.108.104:3000][TaskID: 74] Status: True
    [172.212.108.104:3000][TaskID: 74] Status: Progressing
    [172.212.108.104:3000][TaskID: 74] Logs: Hello from Docker! This message shows that your installation appears to be working correctly. To generate this message, Docker took the following steps: 1. The Docker client contacted the Docker daemon. 2. The Docker daemon pulled the "hello-world" image from the Docker Hub. (amd64) 3. The Docker daemon created a new container from that image which runs the executable that produces the output you are currently reading. 4. The Docker daemon streamed that output to the Docker client, which sent it to your terminal. To try something more ambitious, you can run an Ubuntu container with: $ docker run -it ubuntu bash Share images, automate workflows, and more with a free Docker ID: https://hub.docker.com/ For more examples and ideas, visit: https://docs.docker.com/get-started/

## Simple python

    [172.212.108.104:3000][TaskID: 75] Status: 2024-08-06T13:58:07.000Z
    [172.212.108.104:3000][TaskID: 75] Status: ReplicaSet "dynamic-deployment-mkiqwb-764c6c87" has successfully progressed.
    [172.212.108.104:3000][TaskID: 75] Status: NewReplicaSetAvailable
    [172.212.108.104:3000][TaskID: 75] Status: True
    [172.212.108.104:3000][TaskID: 75] Status: Progressing
    [172.212.108.104:3000][TaskID: 75] Logs: Let's look at some basic math operations one by one 10 + 5 = 15 10 - 5 = 5 10 x 5 = 50 10/5 = 2.0 10^5 = 100 Factorial of 5: 120 Logarithm (base e): 1.0 Sine (of 30 degrees): 0.49999999999999994 Cosine (of 45 degrees): 0.7071067811865476 Tangent (of 60 degrees): 1.7320508075688767 Test Complete!!

## Flog Loader

    [172.212.108.104:3000][TaskID: 76] Status: 2024-08-06T13:59:25.000Z
    [172.212.108.104:3000][TaskID: 76] Status: ReplicaSet "dynamic-deployment-326ael-59f7557547" has successfully progressed.
    [172.212.108.104:3000][TaskID: 76] Status: NewReplicaSetAvailable
    [172.212.108.104:3000][TaskID: 76] Status: True
    [172.212.108.104:3000][TaskID: 76] Status: Progressing
    [172.212.108.104:3000][TaskID: 76] Logs: 133.122.255.57 - wintheiser6332 [06/Aug/2024:13:59:27 +0000] "PUT /next-generation/scale HTTP/1.0" 200 2094 169.217.248.61 - krajcik8327 [06/Aug/2024:13:59:27 +0000] "HEAD /e-enable/repurpose/best-of-breed/web+services HTTP/1.1" 302 1496 219.150.119.243 - - [06/Aug/2024:13:59:27 +0000] "HEAD /paradigms/users/networks/visionary HTTP/1.1" 203 1720 205.181.52.92 - mohr3286 [06/Aug/2024:13:59:27 +0000] "DELETE /revolutionize/repurpose/interfaces HTTP/2.0" 502 25014 221.191.100.146 - - [06/Aug/2024:13:59:27 +0000] "PUT /unleash/transition/innovate/cutting-edge HTTP/1.0" 504 18160 198.99.83.98 - - [06/Aug/2024:13:59:27 +0000] "DELETE /models/sticky/drive/repurpose HTTP/1.0" 503 15005 128.128.104.12 - - [06/Aug/2024:13:59:27 +0000] "PATCH /interactive/synergize/holistic/seize HTTP/1.0" 400 26731 217.66.202.111 - herman1613 [06/Aug/2024:13:59:27 +0000] "POST /ubiquitous/innovate/utilize HTTP/1.0" 302 1253 66.55.244.23 - - [06/Aug/2024:13:59:27 +0000] "GET /evolve HTTP/1.1" 203 982 33.43.231.205 - kuvalis2662 [06/Aug/2024:13:59:27 +0000] "HEAD /engage HTTP/1.1" 204 13898 229.76.176.117 - - [06/Aug/2024:13:59:27 +0000] "DELETE /monetize/orchestrate HTTP/2.0" 302 15400 205.99.160.214 - - [06/Aug/2024:13:59:27 +0000] "POST /synthesize/unleash HTTP/1.1" 301 4818 142.231.87.13 - hettinger8564 [06/Aug/2024:13:59:27 +0000] "DELETE /expedite/e-enable/redefine HTTP/1.1" 204 26815 178.57.223.231 - - [06/Aug/2024:13:59:27 +0000] "POST /content/value-added/bleeding-edge HTTP/1.0" 200 5631 206.138.70.85 - renner6635 [06/Aug/2024:13:59:27 +0000] "PATCH /evolve/iterate/exploit HTTP/1.0" 400 22447 173.97.14.35 - - [06/Aug/2024:13:59:27 +0000] "DELETE /deploy/reinvent HTTP/1.0" 403 255 20.153.146.178 - - [06/Aug/2024:13:59:27 +0000] "DELETE /productize HTTP/1.1" 401 20413 182.140.64.142 - - [06/Aug/2024:13:59:27 +0000] "GET /frictionless/technologies HTTP/1.1" 404 26944 233.158.237.53 - - [06/Aug/2024:13:59:27 +0000] "PATCH /world-class/portals HTTP/1.1" 500 26419 223.117.253.103 - - [06/Aug/2024:13:59:27 +0000] "POST /facilitate HTTP/1.0" 501 3545 6.200.160.191 - schaden2671 [06/Aug/2024:13:59:27 +0000] "GET /vertical/best-of-breed/incentivize HTTP/2.0" 200 9874 113.6.51.228 - - [06/Aug/2024:13:59:27 +0000] "DELETE /syndicate/bleeding-edge/front-end/scale HTTP/1.1" 205 16420 213.247.72.172 - witting8588 [06/Aug/2024:13:59:27 +0000] "PATCH /interactive/seamless/e-business HTTP/1.1" 504 9608 96.250.108.182 - king1218 [06/Aug/2024:13:59:27 +0000] "POST /transition/ubiquitous HTTP/2.0" 403 12068 50.210.127.195 - quigley2241 [06/Aug/2024:13:59:27 +0000] "HEAD /methodologies/deploy HTTP/2.0" 301 27407 6.107.12.88 - - [06/Aug/2024:13:59:27 +0000] "POST /engineer HTTP/1.0" 403 23135 253.76.223.107 - - [06/Aug/2024:13:59:27 +0000] "DELETE /virtual HTTP/1.0" 416 17932 168.150.179.6 - schroeder1815 [06/Aug/2024:13:59:27 +0000] "DELETE /orchestrate/e-business HTTP/1.0" 205 1139 125.189.80.43 - koss1671 [06/Aug/2024:13:59:27 +0000] "POST /frictionless/frictionless HTTP/1.0" 401 18946 112.49.83.207 - ortiz1148 [06/Aug/2024:13:59:27 +0000] "GET /incentivize HTTP/1.0" 200 25120 254.84.164.62 - larson8158 [06/Aug/2024:13:59:27 +0000] "PUT /revolutionary HTTP/2.0" 204 21246 252.79.190.163 - - [06/Aug/2024:13:59:27 +0000] "GET /partnerships/visualize/distributed/utilize HTTP/2.0" 302 26962 14.38.9.69 - gleason5311 [06/Aug/2024:13:59:27 +0000] "DELETE /enable/schemas/orchestrate/incentivize HTTP/1.0" 404 5878 141.210.172.240 - gerhold2783 [06/Aug/2024:13:59:27 +0000] "GET /enterprise/eyeballs HTTP/1.0" 204 13200 143.203.250.79 - bradtke7066 [06/Aug/2024:13:59:27 +0000] "HEAD /architectures/open-source/cutting-edge/enhance HTTP/1.0" 501 9126 81.49.225.99 - pouros3676 [06/Aug/2024:13:59:27 +0000] "POST /grow/synergies HTTP/1.1" 404 9411 241.20.91.187 - - [06/Aug/2024:13:59:27 +0000] "GET /monetize HTTP/1.1" 503 13615 224.106.184.3 - lind2740 [06/Aug/2024:13:59:27 +0000] "PATCH /global/extend HTTP/1.1" 100 24105 154.255.244.193 - - [06/Aug/2024:13:59:27 +0000] "DELETE /systems/interactive HTTP/2.0" 201 17032 87.244.113.77 - cormier6054 [06/Aug/2024:13:59:27 +0000] "HEAD /frictionless/extend/morph HTTP/2.0" 304 25862 46.209.29.76 - hilpert1457 [06/Aug/2024:13:59:27 +0000] "PUT /user-centric/proactive/visionary HTTP/1.1" 204 17537 130.207.222.49 - - [06/Aug/2024:13:59:27 +0000] "PUT /24%2f7/visualize HTTP/2.0" 406 15760 2.210.154.157 - lueilwitz3714 [06/Aug/2024:13:59:27 +0000] "PUT /scale/customized/b2b/cultivate HTTP/1.1" 301 2709 101.195.167.124 - - [06/Aug/2024:13:59:27 +0000] "HEAD /transition/rich/exploit/synergize HTTP/1.1" 205 17096 228.28.71.149 - - [06/Aug/2024:13:59:27 +0000] "HEAD /streamline/scalable/envisioneer/mission-critical HTTP/1.0" 401 5440 194.71.147.240 - - [06/Aug/2024:13:59:27 +0000] "DELETE /interactive/collaborative HTTP/1.1" 203 18018 167.74.143.194 - - [06/Aug/2024:13:59:27 +0000] "GET /next-generation/schemas/solutions/extensible HTTP/2.0" 203 5408 33.79.209.76 - - [06/Aug/2024:13:59:27 +0000] "PATCH /action-items/enable/seamless/mission-critical HTTP/1.0" 502 25038 179.212.18.22 - - [06/Aug/2024:13:59:27 +0000] "PUT /disintermediate/distributed/deliverables/synergistic HTTP/1.1" 500 424 205.60.37.110 - dibbert1188 [06/Aug/2024:13:59:27 +0000] "GET /user-centric/expedite HTTP/1.1" 301 15257 140.188.240.66 - kautzer5027 [06/Aug/2024:13:59:27 +0000] "HEAD /e-commerce/robust HTTP/1.0" 204 26942 26.203.4.91 - - [06/Aug/2024:13:59:27 +0000] "GET /infrastructures HTTP/1.1" 401 14794 184.23.199.30 - - [06/Aug/2024:13:59:27 +0000] "PUT /leading-edge/cultivate/ubiquitous HTTP/1.1" 400 7430 99.172.241.163 - kris7025 [06/Aug/2024:13:59:27 +0000] "POST /vortals HTTP/1.1" 304 23543 41.243.110.180 - - [06/Aug/2024:13:59:27 +0000] "PUT /viral/viral/innovate/b2c HTTP/1.1" 201 26068 229.162.235.184 - - [06/Aug/2024:13:59:27 +0000] "PATCH /integrated/turn-key/cutting-edge/streamline HTTP/1.1" 404 29084 165.188.184.158 - stiedemann2570 [06/Aug/2024:13:59:27 +0000] "GET /bleeding-edge/mission-critical HTTP/1.0" 405 5194 162.225.99.163 - flatley2402 [06/Aug/2024:13:59:27 +0000] "POST /world-class HTTP/1.1" 500 24392 129.170.147.55 - shanahan5205 [06/Aug/2024:13:59:27 +0000] "PATCH /synthesize HTTP/1.0" 500 993 96.109.167.97 - weber5656 [06/Aug/2024:13:59:27 +0000] "POST /experiences/one-to-one/open-source/enable HTTP/1.0" 501 12270 157.75.161.117 - swaniawski3551 [06/Aug/2024:13:59:27 +0000] "PATCH /deploy/convergence/facilitate HTTP/2.0" 503 27450 239.24.222.56 - king4331 [06/Aug/2024:13:59:27 +0000] "PATCH /brand HTTP/2.0" 401 16091 230.207.252.224 - - [06/Aug/2024:13:59:27 +0000] "HEAD /dot-com HTTP/1.1" 201 13833 192.131.65.115 - - [06/Aug/2024:13:59:27 +0000] "PUT /innovative/real-time/envisioneer HTTP/1.1" 501 1991 204.157.213.98 - - [06/Aug/2024:13:59:27 +0000] "GET /reintermediate/empower HTTP/1.1" 500 9699 97.173.9.9 - - [06/Aug/2024:13:59:27 +0000] "GET /visionary/cutting-edge/vertical/integrate HTTP/1.0" 405 1188 51.200.5.26 - - [06/Aug/2024:13:59:27 +0000] "PATCH /real-time/open-source/exploit HTTP/1.0" 200 2050 109.126.6.191 - dietrich6340 [06/Aug/2024:13:59:27 +0000] "PUT /incentivize/best-of-breed/e-business HTTP/2.0" 205 14769 218.200.54.209 - labadie8826 [06/Aug/2024:13:59:27 +0000] "HEAD /architect/world-class/disintermediate/target HTTP/1.1" 503 14097 148.130.36.237 - torp2162 [06/Aug/2024:13:59:27 +0000] "PUT /redefine/monetize HTTP/2.0" 200 6111 84.57.22.131 - treutel8628 [06/Aug/2024:13:59:27 +0000] "POST /e-markets/out-of-the-box HTTP/2.0" 406 22638 29.131.91.145 - - [06/Aug/2024:13:59:27 +0000] "GET /vortals/grow/e-markets/cutting-edge HTTP/1.1" 400 18104 88.70.175.76 - - [06/Aug/2024:13:59:27 +0000] "HEAD /wireless/e-services HTTP/1.1" 503 22700 219.49.140.147 - - [06/Aug/2024:13:59:27 +0000] "PUT /enterprise/compelling/drive/dot-com HTTP/1.0" 204 2283 41.137.212.37 - daniel4557 [06/Aug/2024:13:59:27 +0000] "GET /bleeding-edge/cross-media/real-time/user-centric HTTP/1.1" 201 28351 69.69.232.32 - - [06/Aug/2024:13:59:27 +0000] "PATCH /b2c/content HTTP/1.1" 400 22206 141.58.104.76 - langworth3388 [06/Aug/2024:13:59:27 +0000] "PUT /evolve/technologies HTTP/2.0" 501 26446 28.192.90.164 - - [06/Aug/2024:13:59:27 +0000] "DELETE /24%2f365/exploit HTTP/1.1" 501 17726 221.228.76.123 - ritchie7421 [06/Aug/2024:13:59:27 +0000] "GET /next-generation/enable HTTP/2.0" 403 8782 18.224.134.26 - - [06/Aug/2024:13:59:27 +0000] "POST /synergies/implement/synergistic HTTP/1.0" 302 21571 121.96.21.68 - sipes6211 [06/Aug/2024:13:59:27 +0000] "GET /interfaces/collaborative/platforms/empower HTTP/1.1" 504 18354 105.212.138.108 - - [06/Aug/2024:13:59:27 +0000] "PATCH /visionary HTTP/2.0" 406 16391 186.33.19.210 - - [06/Aug/2024:13:59:27 +0000] "GET /embrace/out-of-the-box/user-centric/web-enabled HTTP/2.0" 205 8941 199.115.194.221 - johnston7278 [06/Aug/2024:13:59:27 +0000] "HEAD /action-items/aggregate HTTP/2.0" 301 28312 38.137.189.82 - - [06/Aug/2024:13:59:27 +0000] "PATCH /supply-chains HTTP/2.0" 304 1192 81.115.147.179 - abbott2381 [06/Aug/2024:13:59:27 +0000] "HEAD /facilitate/extend/orchestrate HTTP/2.0" 500 18241 18.194.16.7 - bogan5472 [06/Aug/2024:13:59:27 +0000] "HEAD /front-end/expedite/visionary HTTP/2.0" 204 27817 183.204.166.90 - gleichner4885 [06/Aug/2024:13:59:27 +0000] "PUT /plug-and-play HTTP/1.0" 304 10059 183.204.44.219 - bogisich8618 [06/Aug/2024:13:59:27 +0000] "GET /magnetic/intuitive HTTP/1.1" 400 9003 75.229.249.168 - - [06/Aug/2024:13:59:27 +0000] "PUT /bricks-and-clicks/visualize/open-source HTTP/1.0" 403 12414 239.4.4.190 - robel1013 [06/Aug/2024:13:59:27 +0000] "GET /plug-and-play/utilize/robust HTTP/2.0" 406 617 198.184.32.195 - fisher2418 [06/Aug/2024:13:59:27 +0000] "POST /envisioneer/deploy/synergize HTTP/2.0" 100 16384 207.161.197.223 - mckenzie1642 [06/Aug/2024:13:59:27 +0000] "PUT /next-generation HTTP/1.0" 400 19030 105.253.177.66 - rohan4730 [06/Aug/2024:13:59:27 +0000] "GET /enterprise HTTP/1.1" 405 12187 104.218.157.142 - - [06/Aug/2024:13:59:27 +0000] "POST /infrastructures/transparent/interactive/bandwidth HTTP/1.0" 501 15912 205.170.254.118 - rempel5050 [06/Aug/2024:13:59:27 +0000] "POST /implement/e-business HTTP/1.1" 400 29954 18.100.44.86 - bergstrom4505 [06/Aug/2024:13:59:27 +0000] "DELETE /open-source/transition HTTP/2.0" 201 8571 67.133.58.195 - bernier4787 [06/Aug/2024:13:59:27 +0000] "PUT /granular/relationships HTTP/1.1" 403 2473 10.117.48.233 - - [06/Aug/2024:13:59:27 +0000] "GET /wireless HTTP/1.0" 501 6537 247.162.177.124 - herman7272 [06/Aug/2024:13:59:27 +0000] "PUT /architect/relationships HTTP/2.0" 205 20722
   
