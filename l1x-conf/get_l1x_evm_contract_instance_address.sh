#!/bin/bash

value=$(yq ".$1.\"$2\".$3" $L1X_CFG_WS_HOME/l1x-conf/config-contract-address-registry.yaml)

# echo $value | sed 's/"//g'
echo $value | tr -d '"' | sed 's/\\//g'
