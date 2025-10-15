#{
    name: "PersistentVolumeClaims (default)",
    matchApiVersion: "v1",
    matchKind: "PersistentVolumeClaim",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata.name
            }
        },
        #{
            title: "Capacity",
            accessor: |obj| {
                obj.spec?.resources?.requests?.storage
            }
        },
        #{
            title: "Access Modes",
            accessor: |obj| {
                obj.spec?.accessModes
            }
        },
        #{
            title: "Status",
            accessor: |obj| {
                let phase = obj.status?.phase;

                if phase == "Bound" {
                    return Text(phase, #{color: "lime"});
                }

                if phase == "Released" {
                    return Text(phase, #{color: "orange"});
                }

                phase
            }
        },
        #{
            title: "Volume",
            accessor: |obj| {
                obj.spec?.volumeName
            }
        },
        #{
            title: "Storage Class",
            accessor: |obj| {
                obj.spec?.storageClassName
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
