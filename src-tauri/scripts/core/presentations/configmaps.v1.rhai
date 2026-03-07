register_resource_presentation(
    ResourcePresentation(#{
        title: "ConfigMaps (default)",
        matcher: |group, version, kind| {
            group == "" && version == "v1" && kind == "ConfigMap"
        },
        columns: [
            ColumnTemplate(#{
                title: "Name",
                render: |obj| {
                    obj.metadata?.name
                }
            }),
            ColumnTemplate(#{
                title: "Namespace",
                render: |obj| {
                    obj.metadata?.namespace
                }
            }),
            ColumnTemplate(#{
                title: "Keys",
                render: |obj| {
                    obj.data?.keys().len() ?? 0
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
