import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import EditorWithToolbar from "../EditorWithToolbar";
import styles from './KubeconfigEditor.module.css';

export interface KubeconfigEditorProps {
    contextSource: KubeContextSource,
    onDirty?: () => void,
    onDirtyCleared?: () => void,
}

export default function KubeconfigEditor({
    contextSource,
    onDirty,
    onDirtyCleared
}: KubeconfigEditorProps) {
    const queryClient = useQueryClient();

    const kubeconfig = useQuery({
        queryKey: ['get_kubeconfig_yaml', contextSource],
        queryFn: () => invoke<string>('get_kubeconfig_yaml', { contextSource }),
    });

    const kubeconfigMutation = useMutation({
        mutationFn: (newYaml: string) => {
            return invoke('write_kubeconfig_yaml', { contextSource, yaml: newYaml });
        },
        onSuccess: async () => {
            // Invalidate all queries because kubeconfig might be shared!
            await queryClient.invalidateQueries({ queryKey: ['getApiServerGitVersion'] });
        }
    });

    async function discard() {
        const refetched = await kubeconfig.refetch();

        if (refetched.isSuccess) {
            return refetched.data;
        }

        throw new Error("Refetch failed");
    }

    if (kubeconfig.isPending || kubeconfig.data === undefined) return;

    return (
        <div className={styles.kubeconfigEditorContainer}>
            <h2>Edit Kubeconfig - {contextSource.source}</h2>
            <div className={styles.editorWrapper}>
                <EditorWithToolbar
                    language="yaml"
                    value={kubeconfig.data}
                    withSaveButton
                    withDiscardButton
                    withWordWrapToggle
                    withWhiteSpaceToggle
                    onSave={(contents) => kubeconfigMutation.mutate(contents)}
                    onDiscard={discard}
                    onDirty={onDirty}
                    onDirtyCleared={onDirtyCleared}
                />
            </div>
        </div>
    );
}
