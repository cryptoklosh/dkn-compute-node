#!/bin/bash

function get_last_log {
    while true; do
        sleep 5m
        cat /root/logs/dkn-compute.log | tail -20 > /root/logs/last_20.log
    done
}

get_last_log &
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

/root/dkn-compute 2>&1 | tee /root/logs/dkn-compute.log
