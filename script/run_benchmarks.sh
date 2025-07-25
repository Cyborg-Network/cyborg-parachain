#!/bin/bash

pallets=(
    #"cumulus_pallet_parachain_system"
    #"cumulus_pallet_xcmp_queue"
    #"orml_oracle"
    #"pallet_balances"
    #"pallet_collator_selection"
    #"pallet_message_queue"
    #"pallet_session"
    #"pallet_sudo"
    #"pallet_timestamp"
    #"pallet_session"
    #"pallet_edge_connect"
    #"pallet_task_management"
    #"pallet_status_aggregator"
    #"pallet_payment"
    "pallet_neuro_zk"
)

# List all unique pallets and extrinsics available for benchmarking
# Command: 
# ./target/release/cyborg-node benchmark pallet --list --chain=dev | tail -n+2 | cut -d',' -f1 | uniq
#
# Note: Execute this script from the project root folder: cyborg-parachain
#



NODE_PATH="./target/release/cyborg-node"
RUNTIME_BLOB="./target/release/wbuild/cyborg-runtime/cyborg_runtime.wasm"
TEMPLATE_PATH=".maintain/frame-weight-template.hbs"
OUTPUT_PATH="./runtime/src/weights"

for pallet in "${pallets[@]}"
do
    echo "Running benchmark for $pallet..."
    OUTPUT_FILE="${OUTPUT_PATH}/${pallet}.rs"

    $NODE_PATH benchmark pallet \
        --runtime=$RUNTIME_BLOB \
        --genesis-builder=runtime \
        --pallet=$pallet \
        --extrinsic="*" \
        --steps=50 \
        --repeat=20 \
        --template=$TEMPLATE_PATH \
        --output=$OUTPUT_FILE

    if [ $? -eq 0 ]; then
        echo "Benchmark completed for $pallet. Weights stored in $OUTPUT_FILE"
    else
        echo "Error:  Benchmark failed for $pallet."
    fi

done

echo "All benchmarks completed."
