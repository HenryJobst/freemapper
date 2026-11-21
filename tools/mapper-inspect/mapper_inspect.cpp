// SPDX-License-Identifier: GPL-3.0-or-later
//
// mapper-inspect: opens a map file (.omap or .ocd) with the importer of
// OpenOrienteering Mapper and prints what it read as JSON. A local check tool,
// not part of the freemapper program. It is built against the Mapper sources
// and therefore falls under their licence (GPL-3.0-or-later). See README.md.
#include <QApplication>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMap>
#include <cstdio>
#include <memory>

#include "core/georeferencing.h"
#include "core/map.h"
#include "core/map_color.h"
#include "core/map_part.h"
#include "core/objects/object.h"
#include "core/symbols/symbol.h"
#include "fileformats/file_format.h"
#include "fileformats/file_format_registry.h"
#include "fileformats/file_import_export.h"
#include "global.h"

using namespace OpenOrienteering;

int main(int argc, char** argv)
{
	QApplication app(argc, argv);
	QCoreApplication::setOrganizationName(QStringLiteral("freemapper-inspect"));
	QCoreApplication::setApplicationName(QStringLiteral("mapper-inspect"));
	doStaticInitializations();
	if (argc != 2)
	{
		std::fputs("usage: mapper-inspect <file.omap|file.ocd>\n", stderr);
		return 2;
	}
	auto const path = QFileInfo(QString::fromLocal8Bit(argv[1])).canonicalFilePath();
	auto const* format = FileFormats.findFormatForFilename(path, &FileFormat::supportsReading);
	if (!format)
	{
		std::fputs("no importer for this file name\n", stderr);
		return 3;
	}
	Map map;
	auto importer = format->makeImporter(path, &map, nullptr);
	if (!importer || !importer->doImport())
	{
		std::fprintf(stderr, "import failed\n");
		if (importer)
			for (auto const& w : importer->warnings())
				std::fprintf(stderr, "%s\n", w.toLocal8Bit().constData());
		return 1;
	}
	QJsonObject out;
	out[QStringLiteral("format")] = QString::fromLatin1(format->id());
	QJsonArray warnings;
	for (auto const& w : importer->warnings())
		warnings.append(w);
	out[QStringLiteral("warnings")] = warnings;

	QJsonArray colors;
	for (int i = 0; i < map.getNumColors(); ++i)
		colors.append(map.getColor(i)->getName());
	out[QStringLiteral("colors")] = colors;

	QMap<int, int> per_symbol;
	int objects = 0;
	for (int p = 0; p < map.getNumParts(); ++p)
	{
		auto const* part = map.getPart(p);
		for (int i = 0; i < part->getNumObjects(); ++i)
		{
			++objects;
			auto const* symbol = part->getObject(i)->getSymbol();
			per_symbol[symbol ? map.findSymbolIndex(symbol) : -1] += 1;
		}
	}
	QJsonArray symbols;
	for (int i = 0; i < map.getNumSymbols(); ++i)
	{
		auto const* s = map.getSymbol(i);
		QJsonObject o;
		o[QStringLiteral("number")] = s->getNumberAsString();
		o[QStringLiteral("name")] = s->getPlainTextName();
		o[QStringLiteral("type")] = int(s->getType());
		o[QStringLiteral("objects")] = per_symbol.value(i, 0);
		symbols.append(o);
	}
	out[QStringLiteral("symbols")] = symbols;
	out[QStringLiteral("objects")] = objects;
	out[QStringLiteral("objects_without_symbol")] = per_symbol.value(-1, 0);
	out[QStringLiteral("scale")] = int(map.getScaleDenominator());

	auto const& g = map.getGeoreferencing();
	QJsonObject geo;
	geo[QStringLiteral("state")] = int(g.getState());  // 0 Local, 1 BrokenGeospatial, 2 Geospatial
	geo[QStringLiteral("scale")] = int(g.getScaleDenominator());
	geo[QStringLiteral("declination")] = g.getDeclination();
	geo[QStringLiteral("grivation")] = g.getGrivation();
	geo[QStringLiteral("crs_id")] = g.getProjectedCRSId();
	geo[QStringLiteral("crs_spec")] = g.getProjectedCRSSpec();
	geo[QStringLiteral("ref_x")] = g.getProjectedRefPoint().x();
	geo[QStringLiteral("ref_y")] = g.getProjectedRefPoint().y();
	geo[QStringLiteral("lat")] = g.getGeographicRefPoint().latitude();
	geo[QStringLiteral("lon")] = g.getGeographicRefPoint().longitude();
	out[QStringLiteral("georef")] = geo;

	std::puts(QJsonDocument(out).toJson(QJsonDocument::Indented).constData());
	return 0;
}
