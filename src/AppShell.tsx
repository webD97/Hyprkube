import { emit } from "@tauri-apps/api/event";
import { ErrorBoundary, FallbackProps } from "react-error-boundary";

import { use, useRef } from "react";
import classes from './AppShell.module.css';
import { DevModeOnly } from "./components/DevModeOnly";
import MegaTabs, { MegaTabsButton } from "./components/MegaTabs";
import { WindowControls } from "./components/WindowControls";
import StatusPanel from "./containers/StatusPanel";
import MegaTabsContext, { MegaTabDefinition } from "./contexts/MegaTabs";
import { useHeadlessTabs } from "./hooks/useHeadlessTabs";
import { Playground } from "./pages/Playground";
import Welcome from "./pages/Welcome";

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
    const megaTabsContext = use(MegaTabsContext)!;

    const {
        closeTab,
        pushTab,
        switchTab,
    } = megaTabsContext;

    const megaTabsOutlet = useRef<HTMLDivElement>(null);

    function openWelcomePage() {
        switchTab(
            pushTab({ title: 'Connect to a cluster', icon: '🔮' }, () => <Welcome />)
        );
    }

    function openDevelopmentPlayground() {
        switchTab(
            pushTab({ title: 'Development playground', icon: '🛝' }, () => <Playground />)
        );
    }

    return (
        <ErrorBoundary fallbackRender={fallbackRender}>
            <div className={classes.container}>
                <header className={classes.header} data-tauri-drag-region>
                    <MegaTabs context={megaTabsContext}
                        onCloseClicked={closeTab}
                        outlet={megaTabsOutlet}
                    >
                        <MegaTabsButton
                            icon="&nbsp;+&nbsp;"
                            title="Open new tab"
                            onClick={openWelcomePage}
                        />
                    </MegaTabs>
                    <section className={classes.right}>
                        <DevModeOnly>
                            <button title="Open development playground" onClick={openDevelopmentPlayground}>🛝</button>
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
    const tabs = useHeadlessTabs<MegaTabDefinition>([
        {
            meta: { title: 'Connect to a cluster', icon: '🔮' },
            render: () => <Welcome />
        }
    ]);

    return (
        <MegaTabsContext.Provider value={tabs}>
            <Layout />
        </MegaTabsContext.Provider>
    );
};

export default AppShell;
