register_resource_presentation(
    ResourcePresentation(#{
        title: "PersistentVolumes (default)",
        matcher: |group, version, kind| {
            group == "" && version == "v1" && kind == "PersistentVolume"
        },
        columns: [
            ColumnTemplate(#{
                title: "Name",
                render: |obj| {
                    obj.metadata.name
                }
            }),
            ColumnTemplate(#{
                title: "Capacity",
                render: |obj| {
                    obj.spec?.capacity?.storage
                }
            }),
            ColumnTemplate(#{
                title: "Access Modes",
                render: |obj| {
                    obj.spec?.accessModes
                }
            }),
            ColumnTemplate(#{
                title: "Reclaim Policy",
                render: |obj| {
                    obj.spec?.persistentVolumeReclaimPolicy
                }
            }),
            ColumnTemplate(#{
                title: "Status",
                render: |obj| {
                    let phase = obj.status?.phase;

                    if phase == "Bound" {
                        return Text(phase, #{color: "lime"});
                    }

                    if phase == "Released" {
                        return Text(phase, #{color: "orange"});
                    }

                    phase
                }
            }),
            ColumnTemplate(#{
                title: "Claim",
                render: |obj| {
                    let namespace = obj.spec?.claimRef?.namespace;
                    let name = obj.spec?.claimRef?.name;

                    `${namespace}/${name}`
                }
            }),
            ColumnTemplate(#{
                title: "Storage Class",
                render: |obj| {
                    obj.spec?.storageClassName
                }
            }),
            ColumnTemplate(#{
                title: "Age",
                render: |obj| {
                    RelativeTime(obj.metadata?.creationTimestamp)
                }
            }),
        ]
    })
);
