fn CopyImageName(obj, container_index) {
    ActionButton(#{
        title: "Copy image name",
        action: |ctx| {
            clipboard::write_text(ctx, obj.spec.containers[container_index].image);
        }
    })
}

fn AttachShell(obj, container_name) {
    ActionButton(#{
        title: "Attach shell",
        action: |ctx| {
            frontend::exec_shell(ctx, obj.metadata?.namespace, obj.metadata?.name, container_name);
        }
    })
}

fn OpenLogs(obj, container_name) {
    ActionButton(#{
        title: "Open logs",
        action: |ctx| {
            frontend::open_logs(ctx, obj.metadata?.namespace, obj.metadata?.name, container_name);
        }
    })
}

register_resource_contextmenu_section(
    MenuSection(#{
        title: "Containers",
        matcher: |group, version, kind| kind == "Pod",
        items: |obj| {
            let containers = obj.spec?.containers?.map(|c| c.name) ?? [];
            containers.map(|name, idx| (
                SubMenu(#{
                    title: name,
                    items: [
                        CopyImageName(obj, idx),
                        AttachShell(obj, name),
                        OpenLogs(obj, name)
                    ]
                })
            ))
        }
    })
);

register_resource_contextmenu_section(
    MenuSection(#{
        title: "Init containers",
        matcher: |group, version, kind| kind == "Pod",
        items: |obj| {
            let init_containers = obj.spec?.initContainers?.map(|c| c.name) ?? [];
            init_containers.map(|name, idx| (
                SubMenu(#{
                    title: name,
                    items: [
                        CopyImageName(obj, idx),
                        AttachShell(obj, name),
                        OpenLogs(obj, name)
                    ]
                })
            ))
        }
    })
);
