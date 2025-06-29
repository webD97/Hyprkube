#{
    name: "ConfigMaps (default)",
    matchApiVersion: "v1",
    matchKind: "ConfigMap",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata?.name
            }
        },
        #{
            title: "Namespace",
            accessor: |obj| {
                obj.metadata?.namespace
            }
        },
        #{
            title: "Keys",
            accessor: |obj| {
                obj.data?.keys().len() ?? 0
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
