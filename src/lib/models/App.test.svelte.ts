import { expect, test } from "vitest";
import { App } from "./App.svelte";
import { applyPatch, compare } from "fast-json-patch";
import { tick } from "svelte";
import { deepCloneOwnProperties } from "$lib/utils";
