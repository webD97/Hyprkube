import { emit } from "@tauri-apps/api/event";
import { ErrorBoundary, FallbackProps } from "react-error-boundary";

import { useContext, useRef } from "react";
import classes from './AppShell.module.css';
import { DevModeOnly } from "./components/DevModeOnly";
import MegaTabs, { MegaTabDefinition, MegaTabsButton } from "./components/MegaTabs";
import { WindowControls } from "./components/WindowControls";
import StatusPanel from "./containers/StatusPanel";
import ApplicationTabsContext from "./contexts/ApplicationTabs";
import { useHeadlessTabs } from "./hooks/useHeadlessTabs";
import ClusterOverview from "./pages/ClusterOverview";
import { Playground } from "./pages/Playground";

function fallbackRender(context: FallbackProps) {
    // Call resetErrorBoundary() to reset the error boundary and retry the render.

    return (
        <div role="alert">
            <p>Something went wrong:</p>
            <pre style={{ color: "red" }}>{JSON.stringify(context.error, undefined, 2)}</pre>
        </div>
    );
}

window.onbeforeunload = function () {
    void emit('frontend-onbeforeunload');
};

const Layout: React.FC = () => {
    const {
        applicationTabs,
        activeApplicationTab,
        pushApplicationTab,
        removeApplicationTab,
        setActiveApplicationTab,
        updateTabMeta
    } = useContext(ApplicationTabsContext)!;

    const megaTabsOutlet = useRef<HTMLDivElement>(null);

    const openClusterExplorer = () => {
        pushApplicationTab({ title: 'Connect to a cluster', icon: '🔮' }, () => <ClusterOverview />);
    };

    if (applicationTabs.length === 0) {
        openClusterExplorer();
    }

    return (
        <ErrorBoundary fallbackRender={fallbackRender}>
            <div className={classes.container}>
                <header className={classes.header} data-tauri-drag-region>
                    <MegaTabs
                        activeTab={activeApplicationTab}
                        setActiveTab={setActiveApplicationTab}
                        onCloseClicked={removeApplicationTab}
                        updateTabMeta={updateTabMeta}
                        tabs={applicationTabs}
                        outlet={megaTabsOutlet}
                    >
                        <MegaTabsButton
                            icon="＋"
                            title="Open new tab"
                            onClick={openClusterExplorer}
                        />
                    </MegaTabs>
                    <section className={classes.right}>
                        <DevModeOnly>
                            <button title="Open development playground"
                                onClick={() => pushApplicationTab({ title: 'Development playground', icon: '🛝' }, () => <Playground />)}
                            >🛝</button>
                        </DevModeOnly>
                        <WindowControls />
                    </section>
                </header>
                <main className={classes.main} ref={megaTabsOutlet}>
                </main>
                <footer className={classes.footer}>
                    <StatusPanel />
                </footer>
            </div>
        </ErrorBoundary>
    );
};

const AppShell: React.FC = () => {
    const [
        applicationTabs,
        activeApplicationTab,
        pushApplicationTab,
        removeApplicationTab,
        setActiveApplicationTab,
        replaceApplicationTab,
        updateTabMeta
    ] = useHeadlessTabs<MegaTabDefinition>();

    return (
        <ApplicationTabsContext.Provider value={{ applicationTabs, activeApplicationTab, pushApplicationTab, removeApplicationTab, setActiveApplicationTab, replaceApplicationTab, updateTabMeta }}>
            <Layout />
        </ApplicationTabsContext.Provider>
    );
};

export default AppShell;
