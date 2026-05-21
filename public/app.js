async function loadData() {
    const response = await fetch('/data.json');
    const data = await response.json();

    document.getElementById('output').textContent =
        JSON.stringify(data, null, 2);
}

document
    .getElementById('btn')
    .addEventListener('click', loadData);