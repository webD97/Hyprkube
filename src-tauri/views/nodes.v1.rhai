#{
    name: "Nodes (default)",
    matchApiVersion: "v1",
    matchKind: "Node",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata?.name
            }
        },
        #{
            title: "OS image",
            accessor: |obj| {
                obj.status?.nodeInfo?.osImage
            }
        },
        #{
            title: "Internal IP",
            accessor: |obj| {
                obj.status?.addresses.find(|address| {
                    address.type == "InternalIP"
                })
                ?.address
            }
        },
        #{
            title: "Architecture",
            accessor: |obj| {
                obj.status?.nodeInfo?.architecture
            }
        },
        #{
            title: "Kubelet version",
            accessor: |obj| {
                obj.status?.nodeInfo?.kubeletVersion
            }
        },
        #{
            title: "Age",
            accessor: |obj| {
                RelativeTime(obj.metadata?.creationTimestamp)
            }
        },
    ]
}
