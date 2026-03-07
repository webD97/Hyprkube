register_resource_presentation(
    ResourcePresentation(#{
        title: "PersistentVolumeClaims (default)",
        matcher: |group, version, kind| {
            group == "" && version == "v1" && kind == "PersistentVolumeClaim"
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
                    obj.spec?.resources?.requests?.storage
                }
            }),
            ColumnTemplate(#{
                title: "Access Modes",
                render: |obj| {
                    obj.spec?.accessModes
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
                title: "Volume",
                render: |obj| {
                    obj.spec?.volumeName
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
