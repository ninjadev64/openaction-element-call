// ==UserScript==
// @name         OpenAction Element Call
// @version      v1.0.0
// @description  Provides basic OpenAction actions for controlling Element Call.
// @author       ninjadev64
// @match        https://call.element.io/*
// @icon         https://www.google.com/s2/favicons?sz=64&domain=element.io
// ==/UserScript==

const openaction_init = () => {
	const openaction_ws = new WebSocket("ws://0.0.0.0:57111");

	const openaction_mic = document.querySelector("[data-testid='incall_mute']");
	const openaction_camera = document.querySelector("[data-testid='incall_videomute']");

	openaction_ws.onmessage = ({ data }) => {
		switch (data) {
			case "toggle_mic": openaction_mic.click(); break;
			case "toggle_camera": openaction_camera.click(); break;
		}
	};

	openaction_mic.addEventListener("click", () => setTimeout(() => {
		if (openaction_mic.className.includes("_on_")) openaction_ws.send("mic_off");
		else openaction_ws.send("mic_on");
	}, 50));

	openaction_camera.addEventListener("click", () => setTimeout(() => {
		if (openaction_camera.className.includes("_on_")) openaction_ws.send("camera_off");
		else openaction_ws.send("camera_on");
	}, 50));
};

const openaction_interval = setInterval(() => {
	if (document.querySelector("[data-testid='incall_mute']")) {
		openaction_init();
		clearInterval(openaction_interval);
	}
}, 50);
