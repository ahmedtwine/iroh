demo:
    #!/usr/bin/env bash
    echo -e "\033[32mTerminal 1: Starting provider...\033[0m"
    cargo run --release --example transfer --all-features -- provide --size 10M > /tmp/iroh_provider.log 2>&1 &
    PROVIDER_PID=$!

    echo -e "\033[32mWaiting for provider to start...\033[0m"
    sleep 8

    NODEADDR=$(grep "Ticket with our home relay and direct addresses:" /tmp/iroh_provider.log -A1 | tail -1)

    echo ""
    echo -e "\033[36mProvider output:\033[0m"
    cat /tmp/iroh_provider.log
    echo ""
    echo -e "\033[33mTerminal 2: Copy and run this command in another terminal:\033[0m"
    echo -e "\033[35mcargo run --release --example transfer --all-features -- fetch \"$NODEADDR\"\033[0m"

    wait $PROVIDER_PID

# Run the mesh demo
mesh:
    cd iroh-mesh && K8S_OPENAPI_ENABLED_VERSION=1.30 cargo run --bin iroh-proxy
