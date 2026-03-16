fn is_restartable(kind) {
    kind in ["StatefulSet", "DaemonSet", "Deployment"]
}

fn has_supported_owner_reference(obj) {
    let owner_references = obj.metadata?.ownerReferences ?? [];
    owner_references.some(|ref| ref.kind == "ReplicaSet" || is_restartable(ref.kind))
}

fn OpenResourceEditor(obj) {
    ActionButton(#{
        title: "Edit YAML",
        action: |ctx| {
            frontend::open_resource_editor(ctx, ResourceRef(obj));
        }
    })
}

fn CopyResourceName(obj) {
    ActionButton(#{
        title: "Copy name",
        action: |ctx| {
            let name = obj.metadata?.name;

            if name != () {
                clipboard::write_text(ctx, name);
            }
        }
    })
}

fn SelectResourceNamespace(obj) {
    ActionButton(#{
        title: "Select namespace",
        action: |ctx| {
            frontend::pick_namespace(ctx, obj.metadata.namespace);
        }
    })
}

fn RestartWorkload(obj) {
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
        let replica_set = owner_references[0];
        let replica_set = kube::get(replica_set.apiVersion, replica_set.kind, obj.metadata.namespace, replica_set.name);

        let owner = replica_set.metadata?.ownerReferences[0];

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
}

fn DeleteResource(obj) {
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
    })
}

register_resource_contextmenu_section(
    MenuSection(#{
        items: |obj| {
            [
                OpenResourceEditor(obj),
                CopyResourceName(obj),
                if obj.metadata?.namespace == () { () } else {
                    SelectResourceNamespace(obj)
                },
                if !has_supported_owner_reference(obj) && !is_restartable(obj.kind) { () } else {
                    RestartWorkload(obj)
                },
                DeleteResource(obj),
            ]
        }
    })
);
