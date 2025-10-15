#{
    name: "PersistentVolumes (default)",
    matchApiVersion: "v1",
    matchKind: "PersistentVolume",
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
                obj.spec?.capacity?.storage
            }
        },
        #{
            title: "Access Modes",
            accessor: |obj| {
                obj.spec?.accessModes
            }
        },
        #{
            title: "Reclaim Policy",
            accessor: |obj| {
                obj.spec?.persistentVolumeReclaimPolicy
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
            title: "Claim",
            accessor: |obj| {
                let namespace = obj.spec?.claimRef?.namespace;
                let name = obj.spec?.claimRef?.name;

                `${namespace}/${name}`
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
