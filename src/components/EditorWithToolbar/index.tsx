import { Editor } from '@monaco-editor/react';
import { editor } from 'monaco-editor';
import { useRef, useState } from 'react';
import Checkbox from '../Checkbox';
import styles from './styles.module.css';

export interface EditorWithToolbarProps {
    value: string,
    language: string,
    readOnly?: boolean,
    withSaveButton?: boolean,
    withDiscardButton?: boolean,
    withWordWrapToggle?: boolean,
    withWhiteSpaceToggle?: boolean,
    onSave?: (contents: string) => void,
    onDiscard?: () => Promise<string>,
    onDirty?: () => void,
    onDirtyCleared?: () => void,
}

export default function EditorWithToolbar({
    value,
    language,
    readOnly = false,
    withSaveButton = false,
    withDiscardButton = false,
    withWordWrapToggle = false,
    withWhiteSpaceToggle = false,
    onSave = () => undefined,
    onDiscard,
    onDirty,
    onDirtyCleared
}: EditorWithToolbarProps) {
    const editorRef = useRef<editor.IStandaloneCodeEditor>(null);
    const modelRef = useRef<string | undefined>(undefined);

    const [whiteSpace, setWhiteSpace] = useState(true);
    const [wordWrap, setWordWrap] = useState(false);

    const [dirty, setDirty] = useState(false);

    function handleFileDirty() {
        onDirty?.();
        setDirty(true);
    }

    function handleFileDirtyCleared() {
        onDirtyCleared?.();
        setDirty(false);
    }

    return (
        <div className={styles.editorContainer}>
            <div className={styles.editorToolbar}>
                <div>
                    {
                        withSaveButton && (
                            <button disabled={!dirty} onClick={() => {
                                if (modelRef.current) onSave(modelRef.current);
                                handleFileDirtyCleared();
                            }}>💾 Save</button>
                        )
                    }
                    {
                        withDiscardButton && (
                            <button disabled={!dirty} onClick={() => {
                                void (async () => {
                                    if (!onDiscard) return;
                                    editorRef.current?.setValue(await onDiscard());
                                    handleFileDirtyCleared();
                                })();
                            }}>⬇️ Reset</button>
                        )
                    }
                </div>
                <div>
                    {
                        withWhiteSpaceToggle && (
                            <Checkbox label="Show whitespace" checked={whiteSpace}
                                onChange={(e) => setWhiteSpace(e.target.checked)}
                            />
                        )
                    }
                    {
                        withWordWrapToggle && (
                            <Checkbox label="Word-wrap" checked={wordWrap}
                                onChange={(e) => setWordWrap(e.target.checked)}
                            />
                        )
                    }
                </div>
            </div>
            <Editor keepCurrentModel
                theme="vs-dark"
                className={styles.editor}
                defaultLanguage={language}
                value={value}
                options={{
                    renderWhitespace: whiteSpace ? "all" : "selection",
                    wordWrap: wordWrap ? 'on' : 'off',
                    readOnly: readOnly,
                    scrollBeyondLastLine: false,
                    minimap: {
                        enabled: false
                    }
                }}
                onMount={(editor) => {
                    editorRef.current = editor;
                    if (modelRef.current) {
                        editor.setValue(modelRef.current);
                    }
                }}
                onChange={(x) => {
                    modelRef.current = x;
                    if (!dirty) handleFileDirty();
                }}
            />
        </div>
    );
}