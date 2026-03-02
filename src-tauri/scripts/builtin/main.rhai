register_resource_contextmenu_section(
    MenuSection(#{
        items: |obj| {
            [
                ActionButton(#{
                    title: "Edit YAML",
                    action: |ctx| {
                        frontend::open_resource_editor(ctx, ResourceRef(obj));
                    }
                }),
                ActionButton(#{
                    title: "Copy name",
                    action: |ctx| {
                        let name = obj.metadata?.name;

                        if name != () {
                            clipboard::write_text(name);
                        }
                    }
                }),
                ActionButton(#{
                    title: "Delete",
                    dangerous: true,
                    action: |ctx| {
                        let api_version = obj.apiVersion;
                        let kind = obj.kind;
                        let namespace = obj.metadata?.namespace;
                        let name = obj.metadata?.name;

                        print(`Deleting ${api_version}/${kind} "${name}" in namespace ${namespace}`);
                        kube::delete(api_version, kind, namespace, name);
                    }
                }),
            ]
        }
    })
);

register_resource_contextmenu_section(
    MenuSection(#{
        title: "Containers",
        matcher: |group, version, kind| kind == "Pod",
        items: |obj| {
            let containers = obj.spec?.containers?.map(|c| c.name) ?? [];
            let init_containers = obj.spec?.initContainers?.map(|c| c.name) ?? [];

            let container_actions = containers.map(|name, idx| (
                SubMenu(#{
                    title: name,
                    items: [
                        ActionButton(#{
                            title: "Copy image name",
                            action: |ctx| {
                                let image = obj.spec.containers[idx].image;
                                print(`Copying image name ${image}!`);
                                clipboard::write_text(image);
                            }
                        }),
                        ActionButton(#{
                            title: "Attach shell",
                            action: |ctx| {
                                print(`Opening shell in container ${name}!`);
                            }
                        }),
                        ActionButton(#{
                            title: "Open logs",
                            action: |ctx| {
                                print(`Opening logs of container ${name}!`);
                            }
                        })
                    ]
                })
            ));

            let init_container_actions = init_containers.map(|name, idx| (
                SubMenu(#{
                    title: name,
                    items: [
                        ActionButton(#{
                            title: "Copy image name",
                            action: |ctx| {
                                clipboard::write_text(obj.spec.containers[idx].image);
                            }
                        }),
                        ActionButton(#{
                            title: "Attach shell",
                            action: |ctx| {
                                print(`Opening shell in container ${name}!`);
                            }
                        }),
                        ActionButton(#{
                            title: "Open logs",
                            action: |ctx| {
                                print(`Opening logs of container ${name}!`);
                            }
                        })
                    ]
                })
            ));

            init_container_actions + container_actions
        }
    })
);

register_resource_contextmenu_section(
    MenuSection(#{
        matcher: |group, version, kind| {
            group == "" && kind in ["ConfigMap", "Secret"]
        },
        items: |obj| {
            let data_keys = obj.data?.keys() ?? [];

            [
                SubMenu(#{
                title: "Copy data",
                    items: data_keys.map(|key| {
                        ActionButton(#{
                            title: key,
                            action: |ctx| {
                                let data = obj.data[key];

                                if obj.kind == "Secret" {
                                    data = base64::decode(data);
                                }

                                clipboard::write_text(data);
                            }
                        })
                    })
                })
            ]
        }
    })
);
