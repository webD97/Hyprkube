#{
    name: "Generic fallback",
    matchApiVersion: "",
    matchKind: "",
    columns: [
        #{
            title: "Namespace",
            accessor: |obj| {
                obj.metadata?.namespace
            }
        },
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata?.name
            }
        },
        #{
            title: "Age",
            accessor: |obj| {
                obj.metadata?.creationTimestamp
            }
        },
    ]
}
