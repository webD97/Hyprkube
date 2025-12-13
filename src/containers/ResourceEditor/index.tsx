import { Editor } from "@monaco-editor/react";
import { type editor } from "monaco-editor";
import { useRef } from "react";
import applyResourceYaml from "../../api/applyResourceYaml";
import getResourceYaml from "../../api/getResourceYaml";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";
import styles from './component.module.css';

export interface ResourceEditorProps {
    fileContent: string,
    contextSource: KubeContextSource,
    currentGvk: Gvk,
    namespace: string,
    name: string
}

const ResourceEditor: React.FC<ResourceEditorProps> = (props) => {
    const {
        fileContent,
        contextSource,
        currentGvk,
        namespace,
        name
    } = props;

    const editorRef = useRef<editor.IStandaloneCodeEditor>(null);

    return (
        <div className={styles.container}>
            <div className={styles.toolbarWrapper}>
                <button
                    onClick={() => {
                        const editor = editorRef.current!;
                        const data = editor.getValue();

                        if (!data) return;

                        applyResourceYaml(contextSource, currentGvk, namespace, name, data)
                            .then(newYaml => {
                                editor.setValue(newYaml);
                            })
                            .catch((e) => {
                                // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                                const {
                                    status,
                                    reason,
                                    message,
                                } = JSON.parse(e as string);

                                const editorValue = editor.getValue().split('\n').filter(line => !line.startsWith('#')).join('\n');
                                editor.setValue(`# ${status} (Reason: ${reason})\n# ${message}\n${editorValue}`);
                            });
                    }}
                >
                    💾 Apply
                </button>
                <button
                    onClick={() => {
                        const editor = editorRef.current!;

                        getResourceYaml(contextSource, currentGvk, namespace, name)
                            .then(yaml => editor.setValue(yaml))
                            .catch(e => alert(JSON.stringify(e)));
                    }}
                >
                    ⭮ Reload
                </button>
            </div>
            <div className={styles.editorWrapper}>
                <Editor
                    height="100%"
                    width="100%"
                    defaultLanguage="yaml"
                    theme="vs-dark"
                    options={{
                        renderWhitespace: "all",
                    }}
                    value={fileContent}
                    onMount={(editor) => {
                        editorRef.current = editor;
                    }}
                />
            </div>
        </div>
    );
}

export default ResourceEditor;