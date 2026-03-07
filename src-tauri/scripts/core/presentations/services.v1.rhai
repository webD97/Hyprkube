register_resource_presentation(
    ResourcePresentation(#{
        title: "Services (default)",
        matcher: |group, version, kind| {
            group == "" && version == "v1" && kind == "Service"
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
                title: "Type",
                render: |obj| {
                    obj.spec?.type
                }
            }),
            ColumnTemplate(#{
                title: "ClusterIP",
                render: |obj| {
                    obj.spec?.clusterIP
                }
            }),
            ColumnTemplate(#{
                title: "ExternalIP",
                render: |obj| {
                    let ips = obj.status?.loadBalancer?.ingress?.map(|ingress| ingress.ip);

                    if ips == () || ips.len == 0 {
                        return "None";
                    }

                    ips
                }
            }),
            ColumnTemplate(#{
                title: "Ports",
                render: |obj| {
                    let ports = obj.spec?.ports?.map(|port| `${port.port}/${port.protocol}`);
                    let display = "";

                    if ports == () {
                        return "None";
                    }

                    for port in ports {
                        if display != "" {
                            display += ", ";
                        }

                        display += port
                    }

                    display
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
