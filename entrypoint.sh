#!/bin/bash

function get_last_log {
    while true; do
        sleep 5m
        cat /root/logs/node_log.log | tail -20 | head -c 5000 > /root/logs/last_20.log
        cat /root/logs/last_20.log > /root/logs/node_log.log
    done
}

get_last_log &
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

/root/dkn-compute 2>&1 | tee /root/logs/node_log.log
