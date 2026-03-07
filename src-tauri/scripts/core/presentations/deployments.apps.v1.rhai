register_resource_presentation(
    ResourcePresentation(#{
        title: "Deployments (default)",
        matcher: |group, version, kind| {
            group == "apps" && version == "v1" && kind == "Deployment"
        },
        columns: [
            ColumnTemplate(#{
                title: "Name",
                render: |obj| {
                    obj.metadata.name
                }
            }),
            ColumnTemplate(#{
                title: "Namespace",
                render: |obj| {
                    obj.metadata.namespace
                }
            }),
            ColumnTemplate(#{
                title: "Pods",
                render: |obj| {
                    let desired = obj.status?.replicas;
                    let actual = desired - (obj.status?.unavailableReplicas ?? 0);

                    `${actual}/${desired}`
                }
            }),
            ColumnTemplate(#{
                title: "Status",
                render: |obj| {
                    let available = obj.status
                        ?.conditions
                        ?.find(|condition| {
                            condition.type == "Available"
                        })
                        ?.status == "True";
                    
                    if available {
                        return Text("Available", #{color: "lime"});
                    }

                    Text("Unavailable", #{color: "crimson"})
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
