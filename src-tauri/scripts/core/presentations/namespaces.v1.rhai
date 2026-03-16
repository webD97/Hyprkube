register_resource_presentation(
    ResourcePresentation(#{
        title: "Namespaces (default)",
        matcher: |group, version, kind| {
            group == "" && version == "v1" && kind == "Namespace"
        },
        columns: [
            ColumnTemplate(#{
                title: "Name",
                render: |obj| {
                    obj.metadata?.name
                }
            }),
            ColumnTemplate(#{
                title: "Phase",
                render: |obj| {
                    let phase = obj.status?.phase;

                    if phase == "Active" {
                        return Text(phase, #{color: "lime"});
                    }

                    if phase == "Terminating" {
                        return Text(phase, #{color: "orange"});
                    }

                    phase
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
