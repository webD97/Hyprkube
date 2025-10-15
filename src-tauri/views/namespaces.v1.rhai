#{
    name: "Namespaces (default)",
    matchApiVersion: "v1",
    matchKind: "Namespace",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata?.name
            }
        },
        #{
            title: "Phase",
            accessor: |obj| {
                let phase = obj.status?.phase;

                if phase == "Active" {
                    return Text(phase, #{color: "lime"});
                }

                if phase == "Terminating" {
                    return Text(phase, #{color: "orange"});
                }

                phase
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
