#{
    name: "Secrets (default)",
    matchApiVersion: "v1",
    matchKind: "Secret",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata.name
            }
        },
        #{
            title: "Namespace",
            accessor: |obj| {
                obj.metadata.namespace
            }
        },
        #{
            title: "Type",
            accessor: |obj| {
                obj.type
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
