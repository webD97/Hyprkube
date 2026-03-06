fn is_restartable(kind) {
    kind in ["StatefulSet", "DaemonSet", "Deployment"]
}

fn has_supported_owner_reference(obj) {
    let owner_references = obj.metadata?.ownerReferences ?? [];

    owner_references.some(|ref| ref.kind == "ReplicaSet")
}

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
                if obj.metadata?.namespace == () { () } else {
                    ActionButton(#{
                        title: "Select namespace",
                        action: |ctx| {
                            frontend::pick_namespace(ctx, obj.metadata.namespace);
                        }
                    })
                },
                if !has_supported_owner_reference(obj) && !is_restartable(obj.kind) { () } else {
                    let owner_references = obj.metadata?.ownerReferences ?? [];
                    let restartee = ();

                    if is_restartable(obj.kind) {
                        restartee = ResourceRef(obj);
                    }
                    else if owner_references.some(|ref| is_restartable(ref.kind)) {
                        let restartable_owners = owner_references.filter(|ref| is_restartable(ref.kind));

                        restartee = ResourceRef(#{
                            api_version: restartable_owners[0].apiVersion,
                            kind: restartable_owners[0].kind,
                            namespace: obj.metadata.namespace,
                            name: restartable_owners[0].name
                        });
                    }
                    else if owner_references[0]?.kind == "ReplicaSet" {
                        print("Deployment");
                        let replica_set = owner_references[0];
                        let replica_set = kube::get(replica_set.apiVersion, replica_set.kind, obj.metadata.namespace, replica_set.name);

                        let owner = replica_set.metadata?.ownerReferences[0];
                        print(owner);

                        restartee = ResourceRef(#{
                            api_version: owner.apiVersion,
                            kind: owner.kind,
                            namespace: obj.metadata.namespace,
                            name: owner.name
                        });
                    }

                    if restartee == () {
                        return ();
                    }

                    ActionButton(#{
                        title: `Restart ${restartee.kind}`,
                        confirm: "Do you really want to restart the workload?",
                        action: |ctx| {
                            kube::rollout_restart(restartee);
                        }
                    })
                },
                ActionButton(#{
                    title: "Delete",
                    dangerous: true,
                    confirm: `Do you really want to delete ${obj.kind} ${obj.metadata.name}? This cannot be undone.`,
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
                                clipboard::write_text(obj.spec.containers[idx].image);
                            }
                        }),
                        ActionButton(#{
                            title: "Attach shell",
                            action: |ctx| {
                                frontend::exec_shell(ctx, obj.metadata?.namespace, obj.metadata?.name, name);
                            }
                        }),
                        ActionButton(#{
                            title: "Open logs",
                            action: |ctx| {
                                frontend::open_logs(ctx, obj.metadata?.namespace, obj.metadata?.name, name);
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
