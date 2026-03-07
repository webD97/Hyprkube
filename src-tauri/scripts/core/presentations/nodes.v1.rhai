register_resource_presentation(
    ResourcePresentation(#{
        title: "Nodes (default)",
        matcher: |group, version, kind| {
            group == "" && version == "v1" && kind == "Node"
        },
        columns: [
            ColumnTemplate(#{
                title: "Name",
                render: |obj| {
                    obj.metadata?.name
                }
            }),
            ColumnTemplate(#{
                title: "OS image",
                render: |obj| {
                    obj.status?.nodeInfo?.osImage
                }
            }),
            ColumnTemplate(#{
                title: "Internal IP",
                render: |obj| {
                    obj.status?.addresses.find(|address| {
                        address.type == "InternalIP"
                    })
                    ?.address
                }
            }),
            ColumnTemplate(#{
                title: "Architecture",
                render: |obj| {
                    obj.status?.nodeInfo?.architecture
                }
            }),
            ColumnTemplate(#{
                title: "Kubelet version",
                render: |obj| {
                    obj.status?.nodeInfo?.kubeletVersion
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
