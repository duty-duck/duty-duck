import common from './common.json'
import showcase from './showcase.json'
import dashboardAuth from './dashboard/auth.json'
import dashboardMenus from './dashboard/menus.json'
import dashboardInputs from './dashboard/inputs.json'
import dashboardHome from './dashboard/home.json'
import dashboardIncidents from './dashboard/incidents.json'
import dashboardMonitors from './dashboard/monitors.json'

export default {
    ...common,
    ...showcase,
    dashboard: {
        ...dashboardAuth,
        ...dashboardMenus,
        ...dashboardInputs,
        ...dashboardHome,
        ...dashboardIncidents,
        ...dashboardMonitors
    }
}