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
                    return ColoredString(phase, "lime");
                }

                if phase == "Terminating" {
                    return ColoredString(phase, "orange");
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
