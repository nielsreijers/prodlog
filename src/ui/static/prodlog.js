// TODO: refactor this later to cache the entry
async function get_prodlog_entry() {
    const uuid = window.location.pathname.split('/').pop();        
    return fetch(`/entry/${uuid}`)
        .then(async response => {
            const data = await response.json();
            if (!response.ok) {
                throw new Error(data.error || 'Failed to load entry');
            }
            return data;
        })
}

async function get_prodlog_diffcontent() {
    const uuid = window.location.pathname.split('/').pop();        
    return fetch(`/diffcontent/${uuid}`)
        .then(async response => {
            const data = await response.json();
            if (!response.ok) {
                throw new Error(data.error || 'Failed to load diff');
            }
            return data;
        })
}

// Export the functions for use in other components
window.prodlog = {
    ...window.prodlog,
    get_prodlog_entry,
    get_prodlog_diffcontent,
};