import { Editor } from "@monaco-editor/react";
import { type editor } from "monaco-editor";
import { useRef } from "react";
import applyResourceYaml from "../../api/applyResourceYaml";
import getResourceYaml from "../../api/getResourceYaml";
import { Gvk } from "../../model/k8s";

export interface ResourceEditorProps {
    fileContent: string,
    clientId: string,
    currentGvk: Gvk,
    namespace: string,
    name: string
}

const ResourceEditor: React.FC<ResourceEditorProps> = (props) => {
    const {
        fileContent,
        clientId,
        currentGvk,
        namespace,
        name
    } = props;

    const editorRef = useRef<editor.IStandaloneCodeEditor>(null);

    return (
        <>
            <button
                onClick={() => {
                    const editor = editorRef.current!;
                    const data = editor.getValue();

                    if (!data) return;

                    applyResourceYaml(clientId, currentGvk, namespace, name, data)
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

                    getResourceYaml(clientId, currentGvk, namespace, name)
                        .then(yaml => editor.setValue(yaml))
                        .catch(e => alert(JSON.stringify(e)));
                }}
            >
                ⭮ Reload
            </button>
            <Editor
                height="600px"
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
        </>
    );
}

export default ResourceEditor;